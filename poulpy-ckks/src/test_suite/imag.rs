//! Multiplication and division by the imaginary unit.

use crate::{CKKSInfos, leveled::api::CKKSImagOps};

use super::helpers::{TestContext, TestPow2Backend as Backend, TestScalar, assert_ct_meta, assert_unary_output_meta};
use poulpy_hal::api::NegacyclicFFT;
use poulpy_hal::api::ScratchOwnedBorrow;

pub fn test_mul_i_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = ctx.alloc_scratch();
    let ct = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let (want_re, want_im) = ctx.want_mul_i();
    let mut ct_res = ctx.alloc_ct(ctx.max_k());
    ctx.module.ckks_mul_i_into(&mut ct_res, &ct, &mut scratch.borrow()).unwrap();
    assert_unary_output_meta("mul_i", &ct_res, &ct);
    ctx.assert_decrypt_precision("mul_i", &ct_res, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_mul_i_smaller_output<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = ctx.alloc_scratch();
    let ct = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let (want_re, want_im) = ctx.want_mul_i();
    let mut ct_res = ctx.alloc_ct(ctx.max_k() - ctx.base2k().as_usize() - 1);
    ctx.module.ckks_mul_i_into(&mut ct_res, &ct, &mut scratch.borrow()).unwrap();
    assert_unary_output_meta("mul_i smaller_output", &ct_res, &ct);
    ctx.assert_decrypt_precision("mul_i", &ct_res, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_mul_i_assign<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = ctx.alloc_scratch();
    let mut ct = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let (want_re, want_im) = ctx.want_mul_i();
    let expected_log_delta = ct.log_delta();
    let expected_log_budget = ct.log_budget();
    ctx.module.ckks_mul_i_assign(&mut ct, &mut scratch.borrow()).unwrap();
    assert_ct_meta("mul_i_assign", &ct, expected_log_delta, expected_log_budget);
    ctx.assert_decrypt_precision("mul_i_assign", &ct, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_div_i_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = ctx.alloc_scratch();
    let ct = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let (want_re, want_im) = ctx.want_div_i();
    let mut ct_res = ctx.alloc_ct(ctx.max_k());
    ctx.module.ckks_div_i_into(&mut ct_res, &ct, &mut scratch.borrow()).unwrap();
    assert_unary_output_meta("div_i", &ct_res, &ct);
    ctx.assert_decrypt_precision("div_i", &ct_res, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_div_i_smaller_output<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = ctx.alloc_scratch();
    let ct = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let (want_re, want_im) = ctx.want_div_i();
    let mut ct_res = ctx.alloc_ct(ctx.max_k() - ctx.base2k().as_usize() - 1);
    ctx.module.ckks_div_i_into(&mut ct_res, &ct, &mut scratch.borrow()).unwrap();
    assert_unary_output_meta("div_i smaller_output", &ct_res, &ct);
    ctx.assert_decrypt_precision("div_i", &ct_res, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_div_i_assign<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = ctx.alloc_scratch();
    let mut ct = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let (want_re, want_im) = ctx.want_div_i();
    let expected_log_delta = ct.log_delta();
    let expected_log_budget = ct.log_budget();
    ctx.module.ckks_div_i_assign(&mut ct, &mut scratch.borrow()).unwrap();
    assert_ct_meta("div_i_assign", &ct, expected_log_delta, expected_log_budget);
    ctx.assert_decrypt_precision("div_i_assign", &ct, &want_re, &want_im, &mut scratch.borrow());
}
