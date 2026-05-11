//! Tests for `CKKSDotProductOps` — inner-product variants.
//!
//! All tests use `n = 3` terms.
//!
//! # Test inventory
//!
//! | Function | Path exercised |
//! |----------|----------------|
//! | [`test_dot_product_ct_aligned`] | ct · ct, aligned |
//! | [`test_dot_product_ct_unaligned`] | ct · ct, one a-side input rescaled |
//! | [`test_dot_product_ct_unaligned_b`] | ct · ct, one b-side input rescaled |
//! | [`test_dot_product_ct_delta_log_delta`] | ct · ct fallback with non-uniform `log_delta` |
//! | [`test_dot_product_ct_smaller_output`] | ct · ct, output narrower than inputs |
//! | [`test_dot_product_pt_vec_aligned`] | ct · ZNX plaintext |
//! | [`test_dot_product_const_aligned`] | ct · ZNX constant |

use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::ScratchOwned,
};

use crate::{layouts::plaintext::CKKSPlaintext, leveled::api::CKKSDotProductOps};

use super::helpers::{TestContext, TestMulBackend as Backend, TestScalar, TestVector};
use poulpy_hal::api::NegacyclicFFT;

const N: usize = 3;
const DELTA_LOG_DELTA: usize = 8;

fn alloc_scratch<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) -> ScratchOwned<BE> {
    let ct_infos = ctx.ct_infos();
    let tsk_infos = ctx.params.tsk_layout();
    let ct_bytes = ctx.module.ckks_dot_product_ct_tmp_bytes(N, &ct_infos, &tsk_infos);
    let pt_bytes = ctx
        .module
        .ckks_dot_product_pt_vec_tmp_bytes(&ct_infos, &ct_infos, &ctx.meta_pt());
    let const_bytes = ctx
        .module
        .ckks_dot_product_pt_const_tmp_bytes(&ct_infos, &ct_infos, &ctx.meta_pt());
    let bytes = ct_bytes.max(pt_bytes).max(const_bytes);
    ScratchOwned::<BE>::alloc(ctx.scratch_size.max(bytes))
}

fn scaled<F: TestScalar>(v: &[F], scale: F) -> Vec<F> {
    v.iter().copied().map(|x| x * scale).collect()
}

fn three_vectors<F: TestScalar>(ctx: &TestContext<impl Backend, F, impl NegacyclicFFT<F>>) -> [(Vec<F>, Vec<F>); 3] {
    let s = F::from_f64(0.5).unwrap();
    [
        (scaled(&ctx.re1, s), scaled(&ctx.im1, s)),
        (scaled(&ctx.re2, s), scaled(&ctx.im2, s)),
        (scaled(&ctx.re1, s), scaled(&ctx.im2, s)),
    ]
}

fn cmul_acc<F: TestScalar>(acc_re: &mut [F], acc_im: &mut [F], a_re: &[F], a_im: &[F], b_re: &[F], b_im: &[F]) {
    for i in 0..acc_re.len() {
        let pr = a_re[i] * b_re[i] - a_im[i] * b_im[i];
        let pi = a_re[i] * b_im[i] + a_im[i] * b_re[i];
        acc_re[i] = acc_re[i] + pr;
        acc_im[i] = acc_im[i] + pi;
    }
}

fn cmul_scalar_acc<F: TestScalar>(acc_re: &mut [F], acc_im: &mut [F], a_re: &[F], a_im: &[F], c_re: F, c_im: F) {
    for i in 0..acc_re.len() {
        let pr = a_re[i] * c_re - a_im[i] * c_im;
        let pi = a_re[i] * c_im + a_im[i] * c_re;
        acc_re[i] = acc_re[i] + pr;
        acc_im[i] = acc_im[i] + pi;
    }
}

pub fn test_dot_product_ct_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let a_vecs = three_vectors(ctx);
    let b_vecs = three_vectors(ctx);

    let m = ctx.re1.len();
    let mut want_re = vec![F::zero(); m];
    let mut want_im = vec![F::zero(); m];
    for i in 0..N {
        cmul_acc(
            &mut want_re,
            &mut want_im,
            &a_vecs[i].0,
            &a_vecs[i].1,
            &b_vecs[i].0,
            &b_vecs[i].1,
        );
    }

    let a_cts: Vec<_> = a_vecs
        .iter()
        .map(|(re, im)| ctx.encrypt(ctx.max_k(), re, im, &mut scratch.borrow()))
        .collect();
    let b_cts: Vec<_> = b_vecs
        .iter()
        .map(|(re, im)| ctx.encrypt(ctx.max_k(), re, im, &mut scratch.borrow()))
        .collect();
    let a_refs: Vec<&_> = a_cts.iter().collect();
    let b_refs: Vec<&_> = b_cts.iter().collect();

    let mut ct_res = ctx.alloc_ct(ctx.max_k());
    ctx.module
        .ckks_dot_product_ct(&mut ct_res, &a_refs, &b_refs, ctx.tsk(), &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision("dot_product_ct_aligned", &ct_res, &want_re, &want_im, &mut scratch.borrow());
}

/// One of the `a`-side ciphertexts is encrypted at a lower `log_budget`.
pub fn test_dot_product_ct_unaligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let a_vecs = three_vectors(ctx);
    let b_vecs = three_vectors(ctx);

    let m = ctx.re1.len();
    let mut want_re = vec![F::zero(); m];
    let mut want_im = vec![F::zero(); m];
    for i in 0..N {
        cmul_acc(
            &mut want_re,
            &mut want_im,
            &a_vecs[i].0,
            &a_vecs[i].1,
            &b_vecs[i].0,
            &b_vecs[i].1,
        );
    }

    let smaller_k = ctx.max_k() - ctx.base2k().as_usize() + 1;
    let a_cts: Vec<_> = a_vecs
        .iter()
        .enumerate()
        .map(|(i, (re, im))| {
            let k = if i == 1 { smaller_k } else { ctx.max_k() };
            ctx.encrypt(k, re, im, &mut scratch.borrow())
        })
        .collect();
    let b_cts: Vec<_> = b_vecs
        .iter()
        .map(|(re, im)| ctx.encrypt(ctx.max_k(), re, im, &mut scratch.borrow()))
        .collect();
    let a_refs: Vec<&_> = a_cts.iter().collect();
    let b_refs: Vec<&_> = b_cts.iter().collect();

    let mut ct_res = ctx.alloc_ct(ctx.max_k());
    ctx.module
        .ckks_dot_product_ct(&mut ct_res, &a_refs, &b_refs, ctx.tsk(), &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision("dot_product_ct_unaligned", &ct_res, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_dot_product_ct_unaligned_b<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let a_vecs = three_vectors(ctx);
    let b_vecs = three_vectors(ctx);

    let m = ctx.re1.len();
    let mut want_re = vec![F::zero(); m];
    let mut want_im = vec![F::zero(); m];
    for i in 0..N {
        cmul_acc(
            &mut want_re,
            &mut want_im,
            &a_vecs[i].0,
            &a_vecs[i].1,
            &b_vecs[i].0,
            &b_vecs[i].1,
        );
    }

    let smaller_k = ctx.max_k() - ctx.base2k().as_usize() + 1;
    let a_cts: Vec<_> = a_vecs
        .iter()
        .map(|(re, im)| ctx.encrypt(ctx.max_k(), re, im, &mut scratch.borrow()))
        .collect();
    let b_cts: Vec<_> = b_vecs
        .iter()
        .enumerate()
        .map(|(i, (re, im))| {
            let k = if i == 1 { smaller_k } else { ctx.max_k() };
            ctx.encrypt(k, re, im, &mut scratch.borrow())
        })
        .collect();
    let a_refs: Vec<&_> = a_cts.iter().collect();
    let b_refs: Vec<&_> = b_cts.iter().collect();

    let mut ct_res = ctx.alloc_ct(ctx.max_k());
    ctx.module
        .ckks_dot_product_ct(&mut ct_res, &a_refs, &b_refs, ctx.tsk(), &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision(
        "dot_product_ct_unaligned_b",
        &ct_res,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}

pub fn test_dot_product_ct_delta_log_delta<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let half = F::from_f64(0.5).unwrap();
    let low_log_delta = ctx.meta().log_delta - DELTA_LOG_DELTA;
    let low_prec = ctx.precision_at(low_log_delta);
    let (a_hi_re, a_hi_im) = ctx.quantized_vector(TestVector::First, ctx.meta().log_delta);
    let (b_hi_re, b_hi_im) = ctx.quantized_vector(TestVector::Second, ctx.meta().log_delta);
    let (b_lo_re, b_lo_im) = ctx.quantized_vector(TestVector::Second, low_log_delta);
    let a_vecs = [
        (scaled(&a_hi_re, half), scaled(&a_hi_im, half)),
        (scaled(&a_hi_re, half), scaled(&a_hi_im, half)),
        (scaled(&a_hi_re, half), scaled(&a_hi_im, half)),
    ];
    let b_vecs = [
        (scaled(&b_lo_re, half), scaled(&b_lo_im, half)),
        (scaled(&b_hi_re, half), scaled(&b_hi_im, half)),
        (scaled(&b_hi_re, half), scaled(&b_hi_im, half)),
    ];

    let m = ctx.re1.len();
    let mut want_re = vec![F::zero(); m];
    let mut want_im = vec![F::zero(); m];
    for i in 0..N {
        cmul_acc(
            &mut want_re,
            &mut want_im,
            &a_vecs[i].0,
            &a_vecs[i].1,
            &b_vecs[i].0,
            &b_vecs[i].1,
        );
    }

    let a_cts: Vec<_> = a_vecs
        .iter()
        .map(|(re, im)| ctx.encrypt(ctx.max_k(), re, im, &mut scratch.borrow()))
        .collect();
    let mut b_cts: Vec<_> = Vec::with_capacity(N);
    b_cts.push(ctx.encrypt_with_prec(
        ctx.max_k() - DELTA_LOG_DELTA,
        &b_vecs[0].0,
        &b_vecs[0].1,
        low_prec,
        &mut scratch.borrow(),
    ));
    for (re, im) in &b_vecs[1..] {
        b_cts.push(ctx.encrypt(ctx.max_k(), re, im, &mut scratch.borrow()));
    }
    let a_refs: Vec<&_> = a_cts.iter().collect();
    let b_refs: Vec<&_> = b_cts.iter().collect();

    let mut ct_res = ctx.alloc_ct(ctx.max_k());
    ctx.module
        .ckks_dot_product_ct(&mut ct_res, &a_refs, &b_refs, ctx.tsk(), &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision_at_log_delta(
        "dot_product_ct_delta_log_delta",
        &ct_res,
        &want_re,
        &want_im,
        low_log_delta,
        &mut scratch.borrow(),
    );
}

pub fn test_dot_product_ct_smaller_output<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let a_vecs = three_vectors(ctx);
    let b_vecs = three_vectors(ctx);

    let m = ctx.re1.len();
    let mut want_re = vec![F::zero(); m];
    let mut want_im = vec![F::zero(); m];
    for i in 0..N {
        cmul_acc(
            &mut want_re,
            &mut want_im,
            &a_vecs[i].0,
            &a_vecs[i].1,
            &b_vecs[i].0,
            &b_vecs[i].1,
        );
    }

    let a_cts: Vec<_> = a_vecs
        .iter()
        .map(|(re, im)| ctx.encrypt(ctx.max_k(), re, im, &mut scratch.borrow()))
        .collect();
    let b_cts: Vec<_> = b_vecs
        .iter()
        .map(|(re, im)| ctx.encrypt(ctx.max_k(), re, im, &mut scratch.borrow()))
        .collect();
    let a_refs: Vec<&_> = a_cts.iter().collect();
    let b_refs: Vec<&_> = b_cts.iter().collect();

    let mut ct_res = ctx.alloc_ct(ctx.max_k() - ctx.base2k().as_usize() - 1);
    ctx.module
        .ckks_dot_product_ct(&mut ct_res, &a_refs, &b_refs, ctx.tsk(), &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision(
        "dot_product_ct smaller_output",
        &ct_res,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}

pub fn test_dot_product_pt_vec_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let a_vecs = three_vectors(ctx);
    let b_vecs = three_vectors(ctx);
    let b_pt_vecs: Vec<_> = b_vecs.iter().map(|(re, im)| ctx.quantized_pt_slots(re, im)).collect();

    let m = ctx.re1.len();
    let mut want_re = vec![F::zero(); m];
    let mut want_im = vec![F::zero(); m];
    for i in 0..N {
        cmul_acc(
            &mut want_re,
            &mut want_im,
            &a_vecs[i].0,
            &a_vecs[i].1,
            &b_pt_vecs[i].0,
            &b_pt_vecs[i].1,
        );
    }

    let a_cts: Vec<_> = a_vecs
        .iter()
        .map(|(re, im)| ctx.encrypt(ctx.max_k(), re, im, &mut scratch.borrow()))
        .collect();
    let pts: Vec<_> = b_vecs.iter().map(|(re, im)| ctx.encode_pt(re, im)).collect();
    let a_refs: Vec<&_> = a_cts.iter().collect();
    let pt_refs: Vec<&_> = pts.iter().collect();

    let mut ct_res = ctx.alloc_ct(ctx.max_k());
    ctx.module
        .ckks_dot_product_pt_vec(&mut ct_res, &a_refs, &pt_refs, &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision(
        "dot_product_pt_vec_aligned",
        &ct_res,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}

pub fn test_dot_product_const_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let a_vecs = three_vectors(ctx);

    let const_coeffs: [f64; 3] = [0.25, 0.125, -0.3125];
    let quantized: Vec<F> = const_coeffs.iter().map(|r| ctx.quantized_const_pt(*r, 0.0).0).collect();

    let m = ctx.re1.len();
    let mut want_re = vec![F::zero(); m];
    let mut want_im = vec![F::zero(); m];
    for i in 0..N {
        cmul_scalar_acc(
            &mut want_re,
            &mut want_im,
            &a_vecs[i].0,
            &a_vecs[i].1,
            quantized[i],
            F::zero(),
        );
    }

    let a_cts: Vec<_> = a_vecs
        .iter()
        .map(|(re, im)| ctx.encrypt(ctx.max_k(), re, im, &mut scratch.borrow()))
        .collect();
    let consts: Vec<CKKSPlaintext<Vec<u8>>> = const_coeffs
        .iter()
        .map(|r| ctx.const_full(Some(*r), None, ctx.meta_pt()))
        .collect();
    let pt_coeffs = vec![0usize; N];

    let a_refs: Vec<&_> = a_cts.iter().collect();
    let const_refs: Vec<&_> = consts.iter().collect();

    let mut ct_res = ctx.alloc_ct(ctx.max_k());
    ctx.module
        .ckks_dot_product_pt_const(&mut ct_res, &a_refs, &const_refs, &pt_coeffs, &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision(
        "dot_product_const_aligned",
        &ct_res,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}
