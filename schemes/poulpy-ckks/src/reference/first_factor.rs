//! Modulus raise with the first `CoeffsToSlots` factor applied on the way.
//!
//! A raised ciphertext is small until its first key switch: integers of a few
//! limbs at the bottom of the Torus. A plaintext product followed by a division
//! keeps it there, at the same precision, so the first factor of the transform
//! consumes no modulus when it runs before any switch
//! ([`DFTPlan::with_first_factor_on_raise`](crate::layouts::DFTPlan::with_first_factor_on_raise)).
//! Each diagonal's term is rotated with no key, which leaves it under the
//! rotated raise secret, then switched to the dense secret by its own key, a
//! switching key whose rows sit `k_offset` bits lower
//! ([`poulpy_core::layouts::GGLWEInfos::k_offset`]): the switch reads the term's
//! digits as balanced integers and lowers them to the bottom of the raised
//! width, which is the raise. The products of all terms are summed in the
//! transform domain ([`poulpy_core::GLWEKeyswitch::glwe_keyswitch_dft_add`]) and
//! rounded once. Under sparse-secret encapsulation the raise secret is the
//! sparse one, and these switches replace the sparse-to-dense one.

use poulpy_core::{
    GLWEBytesOf, GLWECopy, GLWEKeyswitch, GLWENormalize, GLWEShift, GLWESwitchDegree, ScratchArenaTakeGLWE,
    cnv_offset_to_limb_offset, gglwe_product_accumulation_output_size, glwe_left_operand_prepare,
    glwe_left_operand_prepare_tmp_bytes,
    layouts::{
        GGLWEInfos, GLWEInfos, GLWELayout, GLWEToBackendMut, GLWEToBackendRef, IntPolyInfos, LWEInfos, LinearTransformation,
        Rank, prepared::GGLWEPreparedToBackendRef,
    },
    prod_dft_size,
};
use poulpy_hal::{
    api::{
        CnvOps, CnvPVecBytesOf, ScratchArenaTakeCnvPVecL, ScratchArenaTakeVecZnx, ScratchArenaTakeVecZnxDft, VecZnxAddAssign,
        VecZnxAutomorphism, VecZnxCopy, VecZnxDftAddAssign, VecZnxDftBytesOf, VecZnxIdftNormalize, VecZnxIdftNormalizeTmpBytes,
        VecZnxNormalize, VecZnxNormalizeTmpBytes,
    },
    execution::{for_each_with_scratch, worker_count},
    layouts::{
        Backend, CnvPVecLToBackendRef, CnvPVecRToBackendRef, CyclotomicOrder, Module, PrepareHint, ScratchArena, Standard,
        VecZnxDftToBackendMut, VecZnxDftToBackendRef, VecZnxToBackendRef, galois_element,
    },
};

use crate::{
    CKKSCtBounds, CKKSInfos, CKKSResult as Result, GLWEPlaintextMeta, Scale, SetCKKSInfos, ckks_ensure_operand,
    layouts::{
        BootstrappingLinearKeys, CKKSRightOperandPrepared,
        validation::{FirstFactorRaise, check_first_factor_raise, first_factor_key, small_limbs},
    },
    presets::first_factor::first_factor_work,
};

fn layout<A: GLWEInfos>(infos: &A, size: usize) -> GLWELayout {
    GLWELayout {
        n: infos.n(),
        base2k: infos.base2k(),
        k: ((size * infos.base2k().as_usize()) as u32).into(),
        rank: Rank(1),
    }
}

/// The layout of `infos` at the degree of the subring of gap `2^log_gap`.
fn subring_layout<A: GLWEInfos>(infos: &A, log_gap: usize) -> GLWELayout {
    GLWELayout {
        n: (infos.n().as_usize() >> log_gap).into(),
        base2k: infos.base2k(),
        k: infos.k(),
        rank: infos.rank(),
    }
}

/// `res = phi_p(a)` column by column, with no key: `res` is under `phi_p` of
/// the secret of `a`.
fn glwe_automorphism_keyless<BE, R, A>(module: &Module<BE>, res: &mut R, p: i64, a: &A)
where
    BE: Backend,
    Module<BE>: VecZnxAutomorphism<BE>,
    R: GLWEToBackendMut<BE> + GLWEInfos,
    A: GLWEToBackendRef<BE> + GLWEInfos,
{
    assert_eq!((res.size(), res.rank()), (a.size(), a.rank()));
    let a_ref = a.to_backend_ref();
    for col in 0..a.rank().as_usize() + 1 {
        module.vec_znx_automorphism(p, res.to_backend_mut().data_mut(), col, a_ref.data(), col);
    }
    res.set_canonical(a_ref.is_canonical());
}

/// Limbs a diagonal's plaintext holds at radix `base2k`, its encoded width.
fn diagonal_limbs<P: IntPolyInfos>(diagonal: &P, base2k: usize) -> usize {
    diagonal.encoded_k().as_usize().div_ceil(base2k)
}

/// Reference bodies of the required [`CKKSModUpFirstFactorImpl`](crate::oep::CKKSModUpFirstFactorImpl) methods, which
/// [`impl_ckks_mod_up_first_factor_reference_full!`](crate::impl_ckks_mod_up_first_factor_reference_full) forwards to.
pub trait CKKSModUpFirstFactorReference<BE: Backend> {
    fn ckks_mod_up_first_factor_tmp_bytes_reference<Dst, Src, D2S, K>(
        &self,
        log_gap: usize,
        dst_infos: &Dst,
        src_infos: &Src,
        factor: &LinearTransformation<CKKSRightOperandPrepared<BE>>,
        d2s_infos: Option<&D2S>,
        key_infos: &K,
    ) -> usize
    where
        Module<BE>: GLWEKeyswitch<BE>
            + GLWEShift<BE>
            + GLWENormalize<BE>
            + GLWEBytesOf<BE>
            + CnvOps<BE>
            + VecZnxDftBytesOf
            + CnvPVecBytesOf
            + VecZnxNormalizeTmpBytes
            + VecZnxIdftNormalizeTmpBytes,
        Dst: CKKSCtBounds,
        Src: CKKSCtBounds,
        D2S: GGLWEInfos,
        K: GGLWEInfos;

    #[expect(
        clippy::too_many_arguments,
        reason = "the raise takes its scale, the first factor and the gap beside its operands"
    )]
    fn ckks_mod_up_first_factor_reference<Dst, Src, K>(
        &self,
        scale_up: usize,
        log_gap: usize,
        dst: &mut Dst,
        src: &mut Src,
        factor: &LinearTransformation<CKKSRightOperandPrepared<BE>>,
        keys: &K,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Module<BE>: GLWECopy<BE>
            + GLWEShift<BE>
            + GLWEKeyswitch<BE>
            + GLWESwitchDegree<BE>
            + VecZnxAutomorphism<BE>
            + VecZnxCopy<BE>
            + VecZnxAddAssign<BE>
            + VecZnxDftAddAssign<BE>
            + VecZnxNormalize<BE>
            + VecZnxNormalizeTmpBytes
            + VecZnxIdftNormalize<BE>
            + VecZnxIdftNormalizeTmpBytes
            + CnvOps<BE>
            + GLWENormalize<BE>
            + CyclotomicOrder,
        K: BootstrappingLinearKeys<BE>,
        Dst: GLWEToBackendMut<BE> + GLWEToBackendRef<BE> + CKKSCtBounds + SetCKKSInfos,
        Src: GLWEToBackendMut<BE> + GLWEToBackendRef<BE> + CKKSCtBounds + SetCKKSInfos;
}

impl<BE: Backend> CKKSModUpFirstFactorReference<BE> for Module<BE> {
    fn ckks_mod_up_first_factor_tmp_bytes_reference<Dst, Src, D2S, K>(
        &self,
        log_gap: usize,
        dst_infos: &Dst,
        src_infos: &Src,
        factor: &LinearTransformation<CKKSRightOperandPrepared<BE>>,
        d2s_infos: Option<&D2S>,
        key_infos: &K,
    ) -> usize
    where
        Module<BE>: GLWEKeyswitch<BE>
            + GLWEShift<BE>
            + GLWENormalize<BE>
            + GLWEBytesOf<BE>
            + CnvOps<BE>
            + VecZnxDftBytesOf
            + CnvPVecBytesOf
            + VecZnxNormalizeTmpBytes
            + VecZnxIdftNormalizeTmpBytes,
        Dst: CKKSCtBounds,
        Src: CKKSCtBounds,
        D2S: GGLWEInfos,
        K: GGLWEInfos,
    {
        ckks_mod_up_first_factor_tmp_bytes_reference(self, log_gap, dst_infos, src_infos, factor, d2s_infos, key_infos)
    }

    fn ckks_mod_up_first_factor_reference<Dst, Src, K>(
        &self,
        scale_up: usize,
        log_gap: usize,
        dst: &mut Dst,
        src: &mut Src,
        factor: &LinearTransformation<CKKSRightOperandPrepared<BE>>,
        keys: &K,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Module<BE>: GLWECopy<BE>
            + GLWEShift<BE>
            + GLWEKeyswitch<BE>
            + GLWESwitchDegree<BE>
            + VecZnxAutomorphism<BE>
            + VecZnxCopy<BE>
            + VecZnxAddAssign<BE>
            + VecZnxDftAddAssign<BE>
            + VecZnxNormalize<BE>
            + VecZnxNormalizeTmpBytes
            + VecZnxIdftNormalize<BE>
            + VecZnxIdftNormalizeTmpBytes
            + CnvOps<BE>
            + GLWENormalize<BE>
            + CyclotomicOrder,
        K: BootstrappingLinearKeys<BE>,
        Dst: GLWEToBackendMut<BE> + GLWEToBackendRef<BE> + CKKSCtBounds + SetCKKSInfos,
        Src: GLWEToBackendMut<BE> + GLWEToBackendRef<BE> + CKKSCtBounds + SetCKKSInfos,
    {
        ckks_mod_up_first_factor_reference(self, scale_up, log_gap, dst, src, factor, keys, scratch)
    }
}

/// Scratch of [`ckks_mod_up_first_factor_reference`]. `d2s_infos` is the
/// dense-to-sparse key under encapsulation, `key_infos` the layout of the
/// first-factor keys.
pub fn ckks_mod_up_first_factor_tmp_bytes_reference<BE, Dst, Src, D2S, K>(
    module: &Module<BE>,
    log_gap: usize,
    dst_infos: &Dst,
    src_infos: &Src,
    factor: &LinearTransformation<CKKSRightOperandPrepared<BE>>,
    d2s_infos: Option<&D2S>,
    key_infos: &K,
) -> usize
where
    BE: Backend,
    Module<BE>: GLWEKeyswitch<BE>
        + GLWEShift<BE>
        + GLWENormalize<BE>
        + GLWEBytesOf<BE>
        + CnvOps<BE>
        + VecZnxDftBytesOf
        + CnvPVecBytesOf
        + VecZnxNormalizeTmpBytes
        + VecZnxIdftNormalizeTmpBytes,
    Dst: CKKSCtBounds,
    Src: CKKSCtBounds,
    D2S: GGLWEInfos,
    K: GGLWEInfos,
{
    let Some(diagonal) = factor.first_diagonal_plaintext() else {
        return 0;
    };
    let base2k = dst_infos.base2k().as_usize();
    let k_large = dst_infos.k().as_usize();
    let d2s = d2s_infos
        .map_or(0, |key| module.glwe_keyswitch_tmp_bytes(src_infos, src_infos, key))
        .max(if log_gap == 0 {
            0
        } else {
            BE::scratch_aligned(module.glwe_bytes_of_from_infos(&subring_layout(src_infos, log_gap)))
        });
    // A key that does not fit the raise makes it fail before it takes more.
    let Ok(small) = small_limbs("ckks_mod_raise", base2k, k_large, key_infos) else {
        return d2s;
    };
    let small_layout = layout(dst_infos, small);
    let n: usize = dst_infos.n().into();
    let acc_size = gglwe_product_accumulation_output_size::<BE, _, _, _>(
        &layout(dst_infos, k_large.div_ceil(base2k)),
        &small_layout,
        key_infos,
        factor.giant_steps.len(),
    );
    let diagonal_size = diagonal_limbs(diagonal, base2k);
    let (offset, _) = cnv_offset_to_limb_offset(diagonal.encoded_k().as_usize().saturating_sub(diagonal.log_delta()), base2k);
    let product_size = prod_dft_size(small, diagonal_size, offset);
    let carved = 3 * BE::scratch_aligned(module.glwe_bytes_of_from_infos(&small_layout))
        + BE::scratch_aligned(module.cnv_pvec_left_bytes_of(n, 2, small, PrepareHint::Reuse))
        + BE::scratch_aligned(BE::vec_znx_bytes_of(n, 1, small))
        + BE::scratch_aligned(BE::vec_znx_bytes_of(n, 1, acc_size))
        + BE::scratch_aligned(module.vec_znx_dft_bytes_of(n, 2, acc_size));
    let product = BE::scratch_aligned(module.vec_znx_dft_bytes_of(n, 1, product_size))
        + module
            .cnv_apply_dft_tmp_bytes(offset, product_size, small, diagonal_size)
            .max(module.vec_znx_idft_normalize_tmp_bytes(small, product_size));
    let ops = glwe_left_operand_prepare_tmp_bytes(module, small, &small_layout)
        .max(product)
        .max(module.glwe_keyswitch_dft_add_tmp_bytes(acc_size, &small_layout, key_infos))
        .max(module.glwe_keyswitch_dft_tmp_bytes(acc_size, &small_layout, key_infos))
        .max(module.glwe_shift_tmp_bytes(small))
        .max(module.vec_znx_normalize_tmp_bytes())
        .max(module.vec_znx_idft_normalize_tmp_bytes(dst_infos.size(), acc_size));
    d2s.max(carved + BE::scratch_aligned(ops))
}

/// Raises `src` to the precision of `dst` and applies `factor`, the first
/// factor of the transform, on the way: `dst` is what the ordinary raise
/// followed by that factor gives, at the precision of the raise itself.
///
/// `factor` holds one giant step per diagonal, the diagonal rotated back by
/// its own index and no baby step. `keys` gives the dense-to-sparse key under
/// encapsulation and one first-factor key per rotation, all of one layout.
/// `log_gap` is that of the plan: the sparse secret lies in the subring and the raise reads that subring only.
#[expect(
    clippy::too_many_arguments,
    reason = "the raise takes its scale, the first factor and the gap beside its operands"
)]
pub fn ckks_mod_up_first_factor_reference<BE, Dst, Src, K>(
    module: &Module<BE>,
    scale_up: usize,
    log_gap: usize,
    dst: &mut Dst,
    src: &mut Src,
    factor: &LinearTransformation<CKKSRightOperandPrepared<BE>>,
    keys: &K,
    scratch: &mut ScratchArena<'_, BE>,
) -> Result<()>
where
    BE: Backend,
    Module<BE>: GLWECopy<BE>
        + GLWEShift<BE>
        + GLWEKeyswitch<BE>
        + GLWESwitchDegree<BE>
        + VecZnxAutomorphism<BE>
        + VecZnxCopy<BE>
        + VecZnxAddAssign<BE>
        + VecZnxDftAddAssign<BE>
        + VecZnxNormalize<BE>
        + VecZnxNormalizeTmpBytes
        + VecZnxIdftNormalize<BE>
        + VecZnxIdftNormalizeTmpBytes
        + CnvOps<BE>
        + GLWENormalize<BE>
        + CyclotomicOrder,
    K: BootstrappingLinearKeys<BE>,
    Dst: GLWEToBackendMut<BE> + GLWEToBackendRef<BE> + CKKSCtBounds + SetCKKSInfos,
    Src: GLWEToBackendMut<BE> + GLWEToBackendRef<BE> + CKKSCtBounds + SetCKKSInfos,
{
    const OP: &str = "ckks_mod_raise";
    crate::ensure_operand_degrees(OP, module, [dst.n(), src.n()])?;
    let base2k: usize = dst.base2k().as_usize();
    let k_large: usize = dst.k().as_usize();
    ckks_ensure_operand!(
        OP,
        dst.rank().as_usize() == 1 && src.rank().as_usize() == 1,
        "the first factor is applied on a raise of rank-1 ciphertexts only"
    );
    ckks_ensure_operand!(
        OP,
        log_gap == 0 || keys.encapsulation_keys().is_some(),
        "a gap needs the sparse secret of the subring"
    );
    // The dense-to-sparse switch leaves `src.k()` as it is: every check runs
    // before any work.
    let k_small: usize = src.k().as_usize();
    ckks_ensure_operand!(
        OP,
        k_large >= k_small + scale_up,
        "the raise must widen: dst.k ({k_large}) < src.k ({k_small}) + scale_up ({scale_up})"
    );
    // Every key has one layout: one small width for all terms.
    let FirstFactorRaise {
        key_layout,
        small,
        growth,
    } = check_first_factor_raise(
        OP,
        base2k,
        dst.n().as_usize().ilog2() as usize,
        k_large,
        k_small + scale_up,
        factor,
        keys,
    )?;

    let n = dst.n().as_usize();
    let diagonal_size = diagonal_limbs(&factor.giant_steps[0].diagonals[0].plaintext, base2k);
    ckks_ensure_operand!(
        OP,
        factor.giant_steps.iter().all(|g| {
            let rhs = g.diagonals[0].plaintext.cnv();
            rhs.cols() == 1
                && rhs.size() == diagonal_size
                && rhs.n().is_power_of_two()
                && rhs.n() >= BE::MIN_DEGREE
                && n.is_multiple_of(rhs.n())
        }),
        "first-factor diagonals must be plaintexts of the encoded width at a degree dividing the ciphertext degree"
    );

    // The raise runs under the sparse secret when the keys encapsulate it.
    if let Some((dense_to_sparse, _)) = keys.encapsulation_keys() {
        module.glwe_keyswitch_assign(src, &dense_to_sparse.to_backend_ref(), scratch);
    }
    // A secret of `Z[X^g]` decrypts the multiples of `g` on their own: dropping the rest keeps the overflow in the subring.
    if log_gap > 0 {
        let scratch = scratch.borrow();
        let (mut subring, _) = scratch.take_glwe_scratch(&subring_layout(&*src, log_gap));
        module.glwe_switch_degree(&mut subring, &*src);
        module.glwe_switch_degree(src, &subring);
    }
    let k_offset = key_layout.k_offset.as_usize();
    let (_, guard) = first_factor_work(base2k, k_large);
    let small_layout = layout(dst, small);
    // The cyclotomic order of the bootstrap's degree, which the module may exceed.
    let order = crate::operand_cyclotomic_order::<_, Standard>(OP, module, dst.n().as_usize())?;
    let meta = GLWEPlaintextMeta {
        scale: Scale::Log(src.log_delta() + scale_up),
        slots: src.slots(),
        log_sparsity: src.log_sparsity(),
        slot_layout: src.slot_layout(),
    };
    let acc_size = gglwe_product_accumulation_output_size::<BE, _, _, _>(
        &layout(dst, k_large.div_ceil(base2k)),
        &small_layout,
        &key_layout,
        factor.giant_steps.len(),
    );

    {
        let scratch = scratch.borrow();
        let (mut raised, scratch) = scratch.take_glwe_scratch(&small_layout);
        let (mut lowered, scratch) = scratch.take_vec_znx_scratch(n, 1, acc_size);
        let (mut raised_prepared, mut scratch) = scratch.take_cnv_pvec_left_scratch(n, 2, small, PrepareHint::Reuse);

        // The raise: the ciphertext as integers, its last bit `guard` bits (and the
        // lift) above the bottom of the small width, which the keys lower to the
        // bottom of the working width.
        module.glwe_copy(&mut raised, &*src, &mut scratch.borrow());
        module.glwe_rsh_assign(
            small * base2k - k_small - scale_up - guard,
            &mut raised,
            &mut scratch.borrow(),
        );
        glwe_left_operand_prepare(module, &mut raised_prepared, &raised, small * base2k, &mut scratch);
        let raised_prepared = raised_prepared.to_backend_ref();

        // The small products and switches are independent until the sum. Use surplus
        // bootstrap workspace for private partial sums when the executor can run them together.
        // The single-worker path fits the ordinary first-factor scratch query.
        let count = factor.giant_steps.len();
        let factor_keys = factor
            .giant_steps
            .iter()
            .map(|g| Ok(first_factor_key(OP, keys, g.rot)?.to_backend_ref()))
            .collect::<Result<Vec<_>>>()?;
        let (offset, shift) = cnv_offset_to_limb_offset(growth, base2k);
        let product_size = prod_dft_size(small, diagonal_size, offset);
        let local_arrays = 2 * BE::scratch_aligned(BE::vec_znx_bytes_of(n, 2, small));
        let product_bytes = BE::scratch_aligned(BE::vec_znx_dft_bytes_of(n, 1, product_size))
            + module
                .cnv_apply_dft_tmp_bytes(offset, product_size, small, diagonal_size)
                .max(module.vec_znx_idft_normalize_tmp_bytes(small, product_size));
        let local_work = product_bytes
            .max(module.glwe_keyswitch_dft_add_tmp_bytes(acc_size, &small_layout, &key_layout))
            .max(module.glwe_keyswitch_dft_tmp_bytes(acc_size, &small_layout, &key_layout));
        let task_bytes = local_arrays + BE::scratch_aligned(local_work);
        let partial_bytes = BE::scratch_aligned(BE::vec_znx_bytes_of(n, 1, small))
            + BE::scratch_aligned(BE::vec_znx_dft_bytes_of(n, 2, acc_size));
        let finalize = module
            .vec_znx_normalize_tmp_bytes()
            .max(module.vec_znx_idft_normalize_tmp_bytes(dst.size(), acc_size));
        let available = scratch.available();
        let workers = worker_count::<BE::TaskExecutor>(6, count)
            .min(available / (task_bytes + partial_bytes))
            .min(available.saturating_sub(finalize) / partial_bytes)
            .max(1);
        let (accumulators, scratch) = scratch.take_vec_znx_dft_slice_scratch(n, workers, 2, acc_size);
        let (body_sums, mut scratch) = scratch.take_vec_znx_slice_scratch(n, workers, 1, small);
        let mut tasks: Vec<_> = accumulators.into_iter().zip(body_sums).collect();
        let (arenas, _) = scratch.borrow().split(workers, task_bytes);

        for_each_with_scratch::<BE::TaskExecutor, BE, _, _>(&mut tasks, 0, arenas, &|group, (acc, bodies), arena| {
            let (mut product, arena) = arena.borrow().take_glwe_scratch(&small_layout);
            let (mut rotated, mut scratch) = arena.take_glwe_scratch(&small_layout);
            let start = group * count / workers;
            let end = (group + 1) * count / workers;
            for (index, giant) in factor.giant_steps[start..end].iter().enumerate() {
                // Product and division by the diagonal's scale, at the same width.
                let diagonal = &giant.diagonals[0].plaintext;
                product.set_canonical(true);
                {
                    let (mut dft, mut temp) = scratch.borrow().take_vec_znx_dft_scratch(n, 1, product_size);
                    for col in 0..2 {
                        module.cnv_apply_dft(
                            offset,
                            &mut dft,
                            0,
                            &raised_prepared,
                            col,
                            &diagonal.to_backend_ref(),
                            0,
                            1,
                            &mut temp,
                        );
                        module.vec_znx_idft_normalize(
                            product.data_mut(),
                            base2k,
                            small * base2k,
                            shift,
                            col,
                            &mut dft,
                            0,
                            base2k,
                            None,
                            &mut temp,
                        );
                    }
                }

                // The rotation needs no key before the switch.
                let term = if giant.rot == 0 {
                    &product
                } else {
                    glwe_automorphism_keyless(module, &mut rotated, galois_element(giant.rot, order), &product);
                    &rotated
                };

                // The masked part of the switch joins the others in the transform domain;
                // the bodies are summed limb by limb, their integer part kept.
                let key = &factor_keys[start + index];
                let term_ref = term.to_backend_ref();
                if index == 0 {
                    module.glwe_keyswitch_dft(&mut acc.to_backend_mut(), term, key, &mut scratch.borrow());
                    module.vec_znx_copy(bodies, 0, term_ref.data(), 0);
                } else {
                    module.glwe_keyswitch_dft_add(&mut acc.to_backend_mut(), term, key, &mut scratch.borrow());
                    module.vec_znx_add_assign(bodies, 0, term_ref.data(), 0);
                }
            }
        });
        let ((acc, bodies), rest) = tasks.split_first_mut().unwrap();
        for (partial, body) in rest {
            module.vec_znx_add_assign(bodies, 0, &body.to_backend_ref(), 0);
            for col in 0..2 {
                module.vec_znx_dft_add_assign(acc, col, &partial.to_backend_ref(), col);
            }
        }

        // One inverse transform for the whole factor, the bodies lowered as the keys
        // lower the masks, rounded once to the raised precision.
        module.vec_znx_normalize(
            &mut lowered,
            base2k,
            acc_size * base2k,
            -(k_offset as i64),
            0,
            &bodies.to_backend_ref(),
            base2k,
            0,
            &mut scratch.borrow(),
        );
        dst.set_canonical(true);
        let mut dst_ref = dst.to_backend_mut();
        let lowered_ref = lowered.to_backend_ref();
        for col in 0..2 {
            let addend = (col == 0).then_some((&lowered_ref, 0));
            module.vec_znx_idft_normalize(
                dst_ref.data_mut(),
                base2k,
                k_large,
                0,
                col,
                &mut acc.to_backend_mut(),
                col,
                base2k,
                addend,
                &mut scratch.borrow(),
            );
        }
    }

    dst.set_meta(meta);
    Ok(())
}
