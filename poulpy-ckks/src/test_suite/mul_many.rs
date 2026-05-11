//! Tests for `CKKSMulManyOps::ckks_mul_many`.
//!
//! # Test inventory
//!
//! | Function | Path exercised |
//! |----------|----------------|
//! | [`test_mul_many_aligned`] | balanced tree on `n=4` aligned inputs |
//! | [`test_mul_many_two_terms_exact_tmp`] | `n=2` with scratch sized exactly from `ckks_mul_many_tmp_bytes` |
//! | [`test_mul_many_single_smaller_output`] | one input into a narrower output |
//! | [`test_mul_many_odd_tree`] | odd `n=5` exercising the carry-up branch |
//! | [`test_mul_many_unaligned_log_budget`] | one input rescaled by one limb |
//! | [`test_mul_many_smaller_output`] | output narrower than would be needed |

use poulpy_hal::{
    api::{NegacyclicFFT, NegacyclicFFTNew, ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{HostBytesBackend, Module, ScratchOwned},
};

use crate::leveled::api::CKKSMulManyOps;

use super::helpers::{
    TestContextBackend, TestContextModule, TestScalar, alloc_ct, alloc_scratch, assert_decrypt_precision, ckks_encrypt,
    gen_sk_with_raw, gen_tsk, test_vector_1, test_vector_2,
};

use crate::{encoding::reim::Encoder, test_suite::CKKSTestParams};

type Factors<F> = (Vec<(Vec<F>, Vec<F>)>, Vec<F>, Vec<F>);

fn scaled_pair<F: TestScalar>(re: &[F], im: &[F], scale: F) -> (Vec<F>, Vec<F>) {
    let re = re.iter().copied().map(|x| x * scale).collect();
    let im = im.iter().copied().map(|x| x * scale).collect();
    (re, im)
}

fn cmul_assign<F: TestScalar>(acc_re: &mut [F], acc_im: &mut [F], re: &[F], im: &[F]) {
    for i in 0..acc_re.len() {
        let a = acc_re[i];
        let b = acc_im[i];
        let c = re[i];
        let d = im[i];
        acc_re[i] = a * c - b * d;
        acc_im[i] = a * d + b * c;
    }
}

fn build_factors<F: TestScalar>(re1: &[F], im1: &[F], re2: &[F], im2: &[F], n: usize) -> Factors<F> {
    let scale = F::from_f64(0.5).unwrap();
    let (re_a, im_a) = scaled_pair(re1, im1, scale);
    let (re_b, im_b) = scaled_pair(re2, im2, scale);
    let mut factors: Vec<(Vec<F>, Vec<F>)> = Vec::with_capacity(n);
    for k in 0..n {
        factors.push(if k % 2 == 0 {
            (re_a.clone(), im_a.clone())
        } else {
            (re_b.clone(), im_b.clone())
        });
    }
    let m: usize = re1.len();
    let mut want_re: Vec<F> = vec![F::from_f64(1.0).unwrap(); m];
    let mut want_im: Vec<F> = vec![F::zero(); m];
    for (re, im) in factors.iter() {
        cmul_assign(&mut want_re, &mut want_im, re, im);
    }
    (factors, want_re, want_im)
}

pub fn test_mul_many_aligned<BE, F, E>(params: CKKSTestParams, module: &Module<BE>, host_module: &Module<HostBytesBackend>)
where
    BE: TestContextBackend,
    Module<BE>: TestContextModule<BE>,
    F: TestScalar,
    E: NegacyclicFFT<F> + NegacyclicFFTNew<F>,
{
    let m = params.n / 2;
    let encoder = Encoder::<E>::new(m).unwrap();
    let (re1, im1) = test_vector_1::<F>(m);
    let (re2, im2) = test_vector_2::<F>(m);
    let (sk_raw, sk) = gen_sk_with_raw(&params, module, host_module, [0u8; 32]);
    let mut scratch = alloc_scratch(&params, module);
    let tsk = gen_tsk(&params, module, &sk_raw, &mut scratch.borrow());

    let (factors, want_re, want_im) = build_factors(&re1, &im1, &re2, &im2, 4);
    let cts: Vec<_> = factors
        .iter()
        .map(|(re, im)| {
            ckks_encrypt(
                &params,
                module,
                host_module,
                &encoder,
                &sk,
                params.k,
                re,
                im,
                &mut scratch.borrow(),
            )
        })
        .collect();
    let ct_refs: Vec<&_> = cts.iter().collect();
    let mut ct_res = alloc_ct(&params, module, params.k);
    module
        .ckks_mul_many(&mut ct_res, &ct_refs, &tsk, &mut scratch.borrow())
        .unwrap();
    assert_decrypt_precision(
        "mul_many_aligned",
        &params,
        module,
        &encoder,
        &ct_res,
        &sk,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}

pub fn test_mul_many_two_terms_exact_tmp<BE, F, E>(
    params: CKKSTestParams,
    module: &Module<BE>,
    host_module: &Module<HostBytesBackend>,
) where
    BE: TestContextBackend,
    Module<BE>: TestContextModule<BE>,
    F: TestScalar,
    E: NegacyclicFFT<F> + NegacyclicFFTNew<F>,
{
    let n: usize = 2;
    let m = params.n / 2;
    let encoder = Encoder::<E>::new(m).unwrap();
    let (re1, im1) = test_vector_1::<F>(m);
    let (re2, im2) = test_vector_2::<F>(m);
    let (sk_raw, sk) = gen_sk_with_raw(&params, module, host_module, [0u8; 32]);
    let mut setup_scratch = alloc_scratch(&params, module);
    let tsk = gen_tsk(&params, module, &sk_raw, &mut setup_scratch.borrow());

    let (factors, want_re, want_im) = build_factors(&re1, &im1, &re2, &im2, n);
    let cts: Vec<_> = factors
        .iter()
        .map(|(re, im)| {
            ckks_encrypt(
                &params,
                module,
                host_module,
                &encoder,
                &sk,
                params.k,
                re,
                im,
                &mut setup_scratch.borrow(),
            )
        })
        .collect();
    let ct_refs: Vec<&_> = cts.iter().collect();
    let mut ct_res = alloc_ct(&params, module, params.k);

    let tsk_infos = params.tsk_layout();
    let bytes = module.ckks_mul_many_tmp_bytes(n, &ct_res, &tsk_infos);
    let mut op_scratch = ScratchOwned::<BE>::alloc(bytes);
    module
        .ckks_mul_many(&mut ct_res, &ct_refs, &tsk, &mut op_scratch.borrow())
        .unwrap();

    assert_decrypt_precision(
        "mul_many_two_terms_exact_tmp",
        &params,
        module,
        &encoder,
        &ct_res,
        &sk,
        &want_re,
        &want_im,
        &mut setup_scratch.borrow(),
    );
}

pub fn test_mul_many_single_smaller_output<BE, F, E>(
    params: CKKSTestParams,
    module: &Module<BE>,
    host_module: &Module<HostBytesBackend>,
) where
    BE: TestContextBackend,
    Module<BE>: TestContextModule<BE>,
    F: TestScalar,
    E: NegacyclicFFT<F> + NegacyclicFFTNew<F>,
{
    let m = params.n / 2;
    let encoder = Encoder::<E>::new(m).unwrap();
    let (re1, im1) = test_vector_1::<F>(m);
    let (sk_raw, sk) = gen_sk_with_raw(&params, module, host_module, [0u8; 32]);
    let mut scratch = alloc_scratch(&params, module);
    let tsk = gen_tsk(&params, module, &sk_raw, &mut scratch.borrow());

    let ct = ckks_encrypt(
        &params,
        module,
        host_module,
        &encoder,
        &sk,
        params.k,
        &re1,
        &im1,
        &mut scratch.borrow(),
    );
    let ct_refs = vec![&ct];
    let mut ct_res = alloc_ct(&params, module, params.k - params.base2k - 1);
    module
        .ckks_mul_many(&mut ct_res, &ct_refs, &tsk, &mut scratch.borrow())
        .unwrap();
    assert_decrypt_precision(
        "mul_many_single_smaller_output",
        &params,
        module,
        &encoder,
        &ct_res,
        &sk,
        &re1,
        &im1,
        &mut scratch.borrow(),
    );
}

pub fn test_mul_many_odd_tree<BE, F, E>(params: CKKSTestParams, module: &Module<BE>, host_module: &Module<HostBytesBackend>)
where
    BE: TestContextBackend,
    Module<BE>: TestContextModule<BE>,
    F: TestScalar,
    E: NegacyclicFFT<F> + NegacyclicFFTNew<F>,
{
    let m = params.n / 2;
    let encoder = Encoder::<E>::new(m).unwrap();
    let (re1, im1) = test_vector_1::<F>(m);
    let (re2, im2) = test_vector_2::<F>(m);
    let (sk_raw, sk) = gen_sk_with_raw(&params, module, host_module, [0u8; 32]);
    let mut scratch = alloc_scratch(&params, module);
    let tsk = gen_tsk(&params, module, &sk_raw, &mut scratch.borrow());

    let (factors, want_re, want_im) = build_factors(&re1, &im1, &re2, &im2, 5);
    let cts: Vec<_> = factors
        .iter()
        .map(|(re, im)| {
            ckks_encrypt(
                &params,
                module,
                host_module,
                &encoder,
                &sk,
                params.k,
                re,
                im,
                &mut scratch.borrow(),
            )
        })
        .collect();
    let ct_refs: Vec<&_> = cts.iter().collect();
    let mut ct_res = alloc_ct(&params, module, params.k);
    module
        .ckks_mul_many(&mut ct_res, &ct_refs, &tsk, &mut scratch.borrow())
        .unwrap();
    assert_decrypt_precision(
        "mul_many_odd_tree",
        &params,
        module,
        &encoder,
        &ct_res,
        &sk,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}

pub fn test_mul_many_unaligned_log_budget<BE, F, E>(
    params: CKKSTestParams,
    module: &Module<BE>,
    host_module: &Module<HostBytesBackend>,
) where
    BE: TestContextBackend,
    Module<BE>: TestContextModule<BE>,
    F: TestScalar,
    E: NegacyclicFFT<F> + NegacyclicFFTNew<F>,
{
    let n: usize = 4;
    let m = params.n / 2;
    let encoder = Encoder::<E>::new(m).unwrap();
    let (re1, im1) = test_vector_1::<F>(m);
    let (re2, im2) = test_vector_2::<F>(m);
    let (sk_raw, sk) = gen_sk_with_raw(&params, module, host_module, [0u8; 32]);
    let mut scratch = alloc_scratch(&params, module);
    let tsk = gen_tsk(&params, module, &sk_raw, &mut scratch.borrow());

    let (factors, want_re, want_im) = build_factors(&re1, &im1, &re2, &im2, n);
    let smaller_k = params.k - params.base2k + 1;
    let cts: Vec<_> = factors
        .iter()
        .enumerate()
        .map(|(i, (re, im))| {
            let k = if i == 2 { smaller_k } else { params.k };
            ckks_encrypt(&params, module, host_module, &encoder, &sk, k, re, im, &mut scratch.borrow())
        })
        .collect();
    let ct_refs: Vec<&_> = cts.iter().collect();
    let mut ct_res = alloc_ct(&params, module, params.k);
    module
        .ckks_mul_many(&mut ct_res, &ct_refs, &tsk, &mut scratch.borrow())
        .unwrap();
    assert_decrypt_precision(
        "mul_many unaligned_log_budget",
        &params,
        module,
        &encoder,
        &ct_res,
        &sk,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}

pub fn test_mul_many_smaller_output<BE, F, E>(params: CKKSTestParams, module: &Module<BE>, host_module: &Module<HostBytesBackend>)
where
    BE: TestContextBackend,
    Module<BE>: TestContextModule<BE>,
    F: TestScalar,
    E: NegacyclicFFT<F> + NegacyclicFFTNew<F>,
{
    let n: usize = 4;
    let m = params.n / 2;
    let encoder = Encoder::<E>::new(m).unwrap();
    let (re1, im1) = test_vector_1::<F>(m);
    let (re2, im2) = test_vector_2::<F>(m);
    let (sk_raw, sk) = gen_sk_with_raw(&params, module, host_module, [0u8; 32]);
    let mut scratch = alloc_scratch(&params, module);
    let tsk = gen_tsk(&params, module, &sk_raw, &mut scratch.borrow());

    let (factors, want_re, want_im) = build_factors(&re1, &im1, &re2, &im2, n);
    let cts: Vec<_> = factors
        .iter()
        .map(|(re, im)| {
            ckks_encrypt(
                &params,
                module,
                host_module,
                &encoder,
                &sk,
                params.k,
                re,
                im,
                &mut scratch.borrow(),
            )
        })
        .collect();
    let ct_refs: Vec<&_> = cts.iter().collect();
    let mut ct_res = alloc_ct(&params, module, params.k - params.base2k - 1);
    module
        .ckks_mul_many(&mut ct_res, &ct_refs, &tsk, &mut scratch.borrow())
        .unwrap();
    assert_decrypt_precision(
        "mul_many smaller_output",
        &params,
        module,
        &encoder,
        &ct_res,
        &sk,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}
