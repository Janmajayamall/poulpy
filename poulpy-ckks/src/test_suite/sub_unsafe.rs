//! Subtraction tests for the `CKKSSubOpsUnnormalized` API.
//!
//! The safe [`CKKSSubOps`](super::super::sub::CKKSSubOps) path is literally
//! the unnormalized default plus a trailing `glwe_normalize_assign`, so
//! the many path-coverage tests in [`super::sub`] already exercise the
//! shared core for free. These tests only cover what's structurally unique
//! to the unnormalized API:
//!
//! - the unnormalized dispatch reaches the right default helper,
//! - meta (`log_budget`, `log_delta`) is set by the unnormalized op,
//! - calling `.normalize()` on the result recovers a decryptable
//!   ciphertext equivalent to the safe path.
//!
//! One test is kept per distinct kernel family:
//!
//! | Function | Kernel exercised |
//! |----------|------------------|
//! | [`test_sub_ct_aligned_unsafe`] | ct-ct, `glwe_sub` / shift-sub fast path |
//! | [`test_sub_ct_assign_aligned_unsafe`] | ct-ct inplace, `glwe_sub_assign` |
//! | [`test_sub_pt_vec_into_unsafe`] | ct - ZNX plaintext, `VecZnxRshSubBackend` |
//! | [`test_sub_pt_const_into_aligned_unsafe`] | ct - ZNX const, raw `data_mut()[..] -= digit` path |

use poulpy_hal::api::ScratchOwnedBorrow;

use crate::{CKKSInfos, layouts::UnnormalizedCKKSCiphertext, leveled::api::CKKSSubOpsUnnormalized};

use super::helpers::{
    TestContext, TestScalar, TestSubBackend as Backend, TestVector, assert_binary_output_meta, assert_ct_meta,
    assert_unary_output_meta,
};
use poulpy_hal::api::NegacyclicFFT;

pub fn test_sub_ct_aligned_unsafe<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = ctx.alloc_scratch();
    let ct1 = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let ct2 = ctx.encrypt(ctx.max_k(), &ctx.re2, &ctx.im2, &mut scratch.borrow());
    let (want_re, want_im) = ctx.want_sub();
    let mut ct_res = UnnormalizedCKKSCiphertext::new(ctx.alloc_ct(ctx.max_k()));
    ctx.module
        .ckks_sub_into_unnormalized(&mut ct_res, &ct1, &ct2, &mut scratch.borrow())
        .unwrap();
    assert_binary_output_meta("sub_ct_aligned_unsafe", ct_res.as_inner(), &ct1, &ct2);
    let ct_res = ct_res.normalize(&ctx.module, &mut scratch.borrow());
    ctx.assert_decrypt_precision("sub_ct_aligned_unsafe", &ct_res, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_sub_ct_assign_aligned_unsafe<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = ctx.alloc_scratch();
    let ct1_raw = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let ct2 = ctx.encrypt(ctx.max_k(), &ctx.re2, &ctx.im2, &mut scratch.borrow());
    let (want_re, want_im) = ctx.want_sub();
    let expected_log_budget = ct1_raw.log_budget().min(ct2.log_budget());
    let expected_log_delta = ct1_raw.log_delta().max(ct2.log_delta());
    let mut ct1 = UnnormalizedCKKSCiphertext::new(ct1_raw);
    ctx.module
        .ckks_sub_assign_unnormalized(&mut ct1, &ct2, &mut scratch.borrow())
        .unwrap();
    assert_ct_meta(
        "sub_ct_assign_aligned_unsafe",
        ct1.as_inner(),
        expected_log_delta,
        expected_log_budget,
    );
    let ct1 = ct1.normalize(&ctx.module, &mut scratch.borrow());
    ctx.assert_decrypt_precision(
        "sub_ct_assign_aligned_unsafe",
        &ct1,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}

pub fn test_sub_pt_vec_into_unsafe<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = ctx.alloc_scratch();
    let ct1 = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let (pt_re, pt_im) = ctx.pt_vector(TestVector::Second);
    let pt = ctx.encode_pt(&ctx.re2, &ctx.im2);
    let (want_re, want_im) = ctx.want_sub_from(&ctx.re1, &ctx.im1, &pt_re, &pt_im);
    let mut ct_res = UnnormalizedCKKSCiphertext::new(ctx.alloc_ct(ctx.max_k()));
    ctx.module
        .ckks_sub_pt_vec_into_unnormalized(&mut ct_res, &ct1, &pt, &mut scratch.borrow())
        .unwrap();
    assert_unary_output_meta("sub_pt_vec_into_unsafe", ct_res.as_inner(), &ct1);
    let ct_res = ct_res.normalize(&ctx.module, &mut scratch.borrow());
    ctx.assert_decrypt_precision("sub_pt_vec_into_unsafe", &ct_res, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_sub_pt_const_into_aligned_unsafe<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = ctx.alloc_scratch();
    let ct = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let (const_re, const_im) = ctx.add_sub_const();
    let want_re: Vec<F> = ctx.re1.iter().map(|x| *x - const_re).collect();
    let want_im: Vec<F> = ctx.im1.iter().map(|x| *x - const_im).collect();
    let mut ct_res = UnnormalizedCKKSCiphertext::new(ctx.alloc_ct(ctx.max_k()));
    let cst = ctx.add_sub_const_pt();
    ctx.module
        .ckks_sub_pt_const_into_unnormalized(&mut ct_res, &ct, 0, &cst, 0, &mut scratch.borrow())
        .unwrap();
    ctx.module
        .ckks_sub_pt_const_assign_unnormalized(&mut ct_res, ctx.m(), &cst, 1, &mut scratch.borrow())
        .unwrap();
    assert_unary_output_meta("sub_pt_const_into_aligned_unsafe", ct_res.as_inner(), &ct);
    let ct_res = ct_res.normalize(&ctx.module, &mut scratch.borrow());
    ctx.assert_decrypt_precision(
        "sub_pt_const_into_aligned_unsafe",
        &ct_res,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}
