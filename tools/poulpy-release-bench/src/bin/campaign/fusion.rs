//! Offline keys encrypt D * phi(s). Online a switch supplies the mask term
//! and one rank-zero plaintext multiplication supplies D * phi(body).
use super::BE;
use anyhow::{Result, ensure};
use poulpy_ckks::{
    CKKSInfos,
    api::CKKSEncodingHostOps,
    layouts::{CKKSCiphertextOwned, CKKSPlaintextOwned},
    presets::bootstrapping::BootstrappingPreset,
    reference::dft::ckks_dft_matrices_gen,
};
use poulpy_core::layouts::prepared::GGLWEPreparedToBackendRef;
use poulpy_core::{
    GLWEAdd, GLWECopy, GLWEKeyswitch, GLWEMulPlain, GLWENormalize, ScratchArenaTakeGLWE,
    api::{GLWESwitchingKeyEncryptSk, GLWESwitchingKeyPreparedFactory},
    layouts::*,
};
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow, VecZnxAddAssign, VecZnxAutomorphism},
    layouts::{Backend, DataView, Module, ScratchArena, ScratchOwned, Standard, WriterTo, ZnxView, ZnxViewMut, galois_element},
    source::Source,
};

pub struct Term {
    p: i64,
    diagonal: CKKSPlaintextOwned<BE>,
    key: BackendGLWESwitchingKeyPrepared<BE>,
}

pub struct Factor {
    terms: Vec<Term>,
    pub serialized: usize,
    pub prepared: usize,
}

struct Count(usize);
impl std::io::Write for Count {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Factor {
    pub fn new(
        m: &Module<BE>,
        sk: &BackendGLWESecret<BE>,
        preset: &BootstrappingPreset,
        index: usize,
        key_seed: Option<u64>,
        total_factors: usize,
    ) -> Result<Self> {
        let plan = preset.plan().slots_to_coeffs();
        let n = preset.n();
        let radix = preset.base2k();
        let cd = &ckks_dft_matrices_gen::<f64>(plan, preset.log_n())[index];
        let input_k = preset.input_k() - index * plan.coeffs_meta().log_delta();
        let rows = input_k.div_ceil(radix);
        let layout = GLWESwitchingKeyLayout {
            n: n.into(),
            base2k: radix.into(),
            rank_in: Rank(1),
            rank_out: Rank(1),
            dnum: rows.into(),
            k_digit: radix.into(),
            k_offset: 0usize.into(),
            k_aux: (2 * radix).into(),
        };
        ensure!(
            layout.k().as_usize() <= preset.max_dense_modulus(),
            "factor key exceeds dense modulus"
        );
        let passes = total_factors * cd.indexes().len() * 4 * layout.size();
        let failure_bits = 129 + passes.next_power_of_two().ilog2() as usize;
        ensure!(
            radix <= preset.max_backend_base2k::<BE>(129)
                && radix <= Module::<BE>::max_base2k::<Standard>(n, rows.max(1) * layout.size(), failure_bits, false),
            "factor switch exceeds backend accumulation bound"
        );
        let setup_bytes = GLWESwitchingKeyEncryptSk::<BE, Standard>::glwe_switching_key_encrypt_sk_tmp_bytes(m, &layout)
            .max(GLWESwitchingKeyPreparedFactory::<BE, Standard>::glwe_switching_key_prepare_tmp_bytes(m, &layout))
            .max(poulpy_ckks::encoding::ckks_linear_transformation_encode_tmp_bytes::<
                BE,
                _,
                f64,
                Standard,
            >(m, cd.period()));
        let mut arena = ScratchOwned::<BE>::alloc(setup_bytes);
        // One giant group at rotation zero keeps every plaintext in output coordinates.
        let lt = poulpy_ckks::encoding::ckks_linear_transformation_encode_from_diagonals::<_, _, Standard>(
            m,
            radix.into(),
            plan.coeffs_meta(),
            cd,
            LinearTransformationStrategy::Bsgs { giant_step: n / 2 },
            false,
            &mut arena.borrow(),
        )?;
        let mut terms = Vec::new();
        let mut wire = Count(0);
        let mut prepared_bytes = 0;
        let mut seed = [0u8; 32];
        seed[..8].copy_from_slice(&key_seed.unwrap_or(0).to_le_bytes());
        seed[8..16].copy_from_slice(&(index as u64).to_le_bytes());
        seed[16] = 0x91;
        let mut xe = Source::new(seed);
        seed[16] = 0x92;
        let mut xa = Source::new(seed);
        for giant in lt.giant_steps {
            ensure!(giant.rot == 0, "expected unrotated plaintext");
            for d in giant.diagonals {
                let pt = d.plaintext;
                let p = galois_element(d.baby, (2 * n) as i64);
                let mut coeffs = vec![0.0f64; pt.n().as_usize()];
                m.ckks_decode_coeffs_host_into(&pt, &mut coeffs, &mut arena.borrow())?;
                let stride = n / coeffs.len();
                let mut weighted: BackendGLWESecret<BE> = m.glwe_secret_alloc_from_infos(sk);
                let out = weighted.data_mut().at_mut(0, 0);
                let scale = (pt.log_delta() as f64).exp2();
                let taps: Vec<_> = coeffs
                    .iter()
                    .enumerate()
                    .filter_map(|(i, x)| {
                        let value = (x * scale).round() as i64;
                        (value != 0).then_some((i * stride, value))
                    })
                    .collect();
                let bound = taps.iter().map(|(_, v)| v.unsigned_abs()).max().unwrap_or(0) as u128
                    * preset.dense_secret_hamming_weight() as u128;
                ensure!(
                    bound < (1u128 << 51),
                    "weighted secret exceeds the exact host coefficient range"
                );
                for (i, &s) in sk.data().at(0, 0).iter().enumerate().filter(|(_, s)| **s != 0) {
                    let r = i * (p as usize) % (2 * n);
                    for &(j, v) in &taps {
                        let e = (r + j) % (2 * n);
                        out[e % n] += if e < n { s * v } else { -s * v };
                    }
                }
                let mut raw: BackendGLWESwitchingKey<BE> = m.glwe_switching_key_alloc_from_infos(&layout);
                m.glwe_switching_key_encrypt_sk(&mut raw, &weighted, sk, &mut xe, &mut xa, &mut arena.borrow());
                raw.write_to(&mut wire)?;
                let mut key = m.glwe_switching_key_prepared_alloc_from_infos(&raw);
                m.glwe_switching_key_prepare(&mut key, &raw, &mut arena.borrow());
                prepared_bytes += BE::len_bytes_ref(GGLWEPreparedToBackendRef::<BE>::to_backend_ref(&key).data().data());
                terms.push(Term { p, diagonal: pt, key });
            }
        }
        Ok(Self {
            terms,
            serialized: wire.0,
            prepared: prepared_bytes,
        })
    }

    pub fn apply(&self, m: &Module<BE>, ct: &mut CKKSCiphertextOwned<BE>, scratch: &mut ScratchArena<'_, BE>) {
        use poulpy_core::gglwe_product_accumulation_output_size;
        use poulpy_hal::{
            api::{
                ScratchArenaTakeScalarZnx, ScratchArenaTakeSvpPPol, ScratchArenaTakeVecZnxDft, SvpPPolPrepare,
                VecZnxDftAddAssign, VecZnxDftApply, VecZnxDftAutomorphism, VecZnxDftAutomorphismPlan, VecZnxDftZero,
                VecZnxIdftNormalize, VecZnxIdftNormalizeTmpBytes, VmpApplyDftToDftAdd, VmpApplyDftToDftAddTmpBytes,
            },
            layouts::{PrepareHint, ScalarZnxToBackendRef, VecZnxDftToBackendRef},
        };
        use rayon::prelude::*;

        let n = ct.n().as_usize();
        let radix = ct.base2k().as_usize();
        let out_k = ct.k().as_usize() - self.terms[0].diagonal.log_delta();
        let output = GLWELayout {
            n: ct.n(),
            k: out_k.into(),
            base2k: ct.base2k(),
            rank: ct.rank(),
        };
        let count = self.terms.len();
        let key = &self.terms[0].key;
        let acc_size = gglwe_product_accumulation_output_size::<BE, _, _, _>(&output, ct, key, count);
        // Cover both the switch and the body product accumulated for each term. There are
        // at most log2(n) transform factors. Reserve half the failure budget for fusion.
        let failure_bits = 129 + (n.ilog2() as usize * count * 4 * key.size()).next_power_of_two().ilog2() as usize;
        let input_size = ct.size();
        let max_radix = Module::<BE>::max_base2k::<Standard>(n, count * (input_size + 1), failure_bits, false);
        let max_degree = self.terms.iter().map(|t| t.diagonal.n().as_usize()).max().unwrap();
        let diagonal_bytes = BE::scratch_aligned(BE::scalar_znx_bytes_of(max_degree, 1))
            + BE::scratch_aligned(BE::svp_ppol_bytes_of(max_degree, 1, PrepareHint::OneShot));
        let acc_bytes = BE::scratch_aligned(BE::vec_znx_dft_bytes_of(n, 2, acc_size));
        let op_bytes = m.vmp_apply_dft_to_dft_add_tmp_bytes(acc_size, input_size, key.dnum().as_usize(), 1, 2, key.size());
        let per_worker =
            BE::scratch_aligned(BE::vec_znx_dft_bytes_of(n, 1, input_size)) + diagonal_bytes.max(BE::scratch_aligned(op_bytes));
        let input_bytes = BE::scratch_aligned(BE::vec_znx_dft_bytes_of(n, 2, input_size));
        let available = scratch.available().saturating_sub(input_bytes);
        let groups = count
            .min(rayon::current_num_threads())
            .min(available / (acc_bytes + per_worker));
        if self
            .terms
            .iter()
            .any(|t| t.diagonal.n().as_usize() < BE::MIN_DEGREE || t.diagonal.encoded_k().as_usize() > 64)
            || radix > max_radix
            || groups == 0
            || available - groups * acc_bytes < m.vec_znx_idft_normalize_tmp_bytes(output.size(), acc_size)
        {
            return self.apply_separate(m, ct, scratch);
        }

        if !GLWEToBackendRef::<BE>::is_canonical(ct) {
            m.glwe_normalize_assign(ct, scratch);
        }
        let (mut input, work) = scratch.borrow().take_vec_znx_dft_scratch(n, 2, ct.size());
        let src = GLWEToBackendRef::<BE>::to_backend_ref(ct);
        for col in 0..2 {
            m.vec_znx_dft_apply(1, 0, &mut input, col, src.data(), col);
        }
        let (mut partials, mut work) = work.take_vec_znx_dft_slice_scratch(n, groups, 2, acc_size);
        let (arenas, _) = work.borrow().split(groups, per_worker);
        partials.par_iter_mut().zip(arenas).enumerate().for_each(|(g, (acc, arena))| {
            let (mut mask, mut arena) = arena.take_vec_znx_dft_scratch(n, 1, input_size);
            for col in 0..2 {
                m.vec_znx_dft_zero(acc, col);
            }
            for term in &self.terms[g * count / groups..(g + 1) * count / groups] {
                // Gadget digits are whole limbs. Applying the Galois permutation after the
                // two forward transforms shares their cost across every diagonal.
                let rotation = <Module<BE> as VecZnxDftAutomorphismPlan<BE>>::vec_znx_dft_automorphism_plan(m, n, term.p);
                let input = input.to_backend_ref();
                // A subring NTT evaluation repeats at consecutive full-ring frequencies.
                // Read the diagonal at its own degree and accumulate each weighted
                // Galois image directly into the switch output.
                let degree = term.diagonal.n().as_usize();
                {
                    let (mut scalar, temp) = arena.borrow().take_scalar_znx_scratch(degree, 1);
                    term.diagonal.decode_vec_i64(scalar.at_mut(0, 0), term.diagonal.encoded_k());
                    let (mut diagonal, _) = temp.take_svp_ppol_scratch::<Standard>(degree, 1, PrepareHint::OneShot);
                    m.svp_ppol_prepare(&mut diagonal, 0, &scalar.to_backend_ref(), 0);
                    let columns = if term.p == 1 { 2 } else { 1 };
                    for limb in 0..input_size.min(acc_size) {
                        for col in 0..columns {
                            Self::add_weighted_rotation(
                                acc.at_mut(col, limb),
                                input.at(col, limb),
                                &rotation.perm,
                                diagonal.raw(),
                            );
                        }
                    }
                }
                if term.p != 1 {
                    m.vec_znx_dft_automorphism_with_plan(&rotation, &mut mask, 0, &input, 1);
                    let key = GGLWEPreparedToBackendRef::<BE>::to_backend_ref(&term.key);
                    m.vmp_apply_dft_to_dft_add(acc, &mask.to_backend_ref(), key.data(), 0, &mut arena);
                }
            }
        });
        let (acc, rest) = partials.split_first_mut().unwrap();
        for partial in rest {
            for col in 0..2 {
                m.vec_znx_dft_add_assign(acc, col, &partial.to_backend_ref(), col);
            }
        }
        ct.set_k(out_k.into());
        GLWEToBackendMut::<BE>::set_canonical(ct, true);
        GLWEToBackendMut::<BE>::set_noise(ct, None);
        let mut dst = GLWEToBackendMut::<BE>::to_backend_mut(ct);
        for col in 0..2 {
            m.vec_znx_idft_normalize(dst.data_mut(), radix, out_k, 0, col, acc, col, radix, None, &mut work);
        }
    }

    /// The AVX backend keeps four canonical 30-bit residues per NTT coefficient.
    #[inline]
    fn add_weighted_rotation(
        dst: &mut [<BE as Backend>::DftWord],
        src: &[<BE as Backend>::DftWord],
        permutation: &[u32],
        diagonal: &[<BE as Backend>::DftWord],
    ) {
        use poulpy_cpu_portable::kernels::ntt4x30::primes::Primes30;
        use poulpy_hal::layouts::PrimeSet;

        let log_stride = (src.len() / diagonal.len()).ilog2();
        for (i, (out, &j)) in dst.iter_mut().zip(permutation).enumerate() {
            let a = src[j as usize].0;
            let d = diagonal[i >> log_stride].0;
            for prime in 0..4 {
                let sum = u64::from(out.0[prime]) + u64::from(a[prime]) * u64::from(d[prime]);
                // For sum < 2^60 these two truncations underestimate sum/q by at most two.
                // Both factors in the quotient multiply fit u32, including the fixed reciprocal.
                let q = Primes30::Q[prime];
                let reciprocal = (1u64 << 61) / u64::from(q);
                let quotient = ((sum >> 29) * reciprocal) >> 32;
                let r = (sum as u32).wrapping_sub((quotient as u32).wrapping_mul(q));
                let r = r.min(r.wrapping_sub(q));
                out.0[prime] = r.min(r.wrapping_sub(q));
            }
        }
    }

    fn apply_separate(&self, m: &Module<BE>, ct: &mut CKKSCiphertextOwned<BE>, scratch: &mut ScratchArena<'_, BE>) {
        let full = GLWELayout {
            n: ct.n(),
            base2k: ct.base2k(),
            rank: Rank(1),
            k: ct.k(),
        };
        let narrow = GLWELayout {
            k: (ct.k().as_usize() - self.terms[0].diagonal.log_delta()).into(),
            ..full
        };
        let (mut mask, work) = scratch.borrow().take_glwe_scratch(&full);
        let (mut body, work) = work.take_glwe_scratch(&GLWELayout { rank: Rank(0), ..full });
        let (mut switched, work) = work.take_glwe_scratch(&full);
        let (mut bp, work) = work.take_glwe_scratch(&GLWELayout { rank: Rank(0), ..full });
        let (mut sum, mut work) = work.take_glwe_scratch(&full);
        // Scratch is reused and can contain live data from preceding steps.
        mask.data_mut().raw_mut().fill(0);
        sum.data_mut().raw_mut().fill(0);
        for t in &self.terms {
            let src = GLWEToBackendRef::<BE>::to_backend_ref(ct);
            m.vec_znx_automorphism(t.p, mask.data_mut(), 1, src.data(), 1);
            m.vec_znx_automorphism(t.p, body.data_mut(), 0, src.data(), 0);
            mask.set_canonical(false);
            body.set_canonical(false);
            m.glwe_keyswitch(
                &mut switched,
                &mask,
                &GGLWEPreparedToBackendRef::to_backend_ref(&t.key),
                &mut work,
            );
            m.glwe_mul_plain(t.diagonal.encoded_k().as_usize(), &mut bp, &body, &t.diagonal, &mut work);
            m.vec_znx_add_assign(switched.data_mut(), 0, GLWEToBackendRef::<BE>::to_backend_ref(&bp).data(), 0);
            switched.set_canonical(false);
            m.glwe_add_assign(&mut sum, &switched);
            m.glwe_normalize_assign(&mut sum, &mut work);
        }
        ct.set_k(narrow.k);
        m.glwe_copy(ct, &sum, &mut work);
        GLWEToBackendMut::<BE>::set_noise(ct, None);
    }
}

#[cfg(test)]
mod tests {
    use super::Factor;
    use poulpy_cpu_portable::kernels::ntt4x30::primes::Primes30;
    use poulpy_hal::layouts::{CrtWord, PrimeSet};

    #[test]
    fn weighted_subring_rotation_matches_modular_product_sum() {
        let values = [0, 1, 2, (1 << 29) - 1, 1 << 29].map(|v| CrtWord::<Primes30, u32>([v; 4]));
        let near_q = [3, 2, 1].map(|delta| CrtWord(Primes30::Q.map(|q| q - delta)));
        let src = [values.as_slice(), near_q.as_slice()].concat();
        let permutation = [3, 7, 1, 0, 4, 5, 2, 6];
        for &initial in &src {
            for i in 0..src.len() {
                let diagonal = [src[i], src[(i + 1) % src.len()]];
                let mut dst = [initial; 8];
                Factor::add_weighted_rotation(&mut dst, &src, &permutation, &diagonal);
                for (k, &j) in permutation.iter().enumerate() {
                    for (p, q) in Primes30::Q.into_iter().enumerate() {
                        let product = u64::from(src[j as usize].0[p]) * u64::from(diagonal[k / 4].0[p]);
                        let expected = (u64::from(initial.0[p]) + product) % u64::from(q);
                        assert_eq!(u64::from(dst[k].0[p]), expected);
                    }
                }
            }
        }
    }
}
