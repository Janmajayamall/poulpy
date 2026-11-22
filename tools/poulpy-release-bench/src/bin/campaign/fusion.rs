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
