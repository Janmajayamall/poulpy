//! Tests for `CKKSMulSubOps` — fused `dst -= a · b` variants.
//!
//! # Test inventory
//!
//! | Function | Path exercised |
//! |----------|----------------|
//! | [`test_mul_sub_ct_aligned`] | ct-ct, all operands aligned |
//! | [`test_mul_sub_ct_unaligned_dst`] | ct-ct with `dst` at a lower `log_budget` |
//! | [`test_mul_sub_pt_vec_aligned`] | ZNX plaintext, aligned |
//! | [`test_mul_sub_pt_vec_into_delta_log_delta`] | ZNX plaintext at lower `log_delta` |
//! | [`test_mul_sub_pt_const_into_aligned`] | ZNX constant, aligned |
//! | [`test_mul_sub_pt_const_zero_preserves_dst_meta`] | ZNX zero constant no-op |

use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::ScratchOwned,
};

use crate::{CKKSInfos, CKKSMeta, leveled::api::CKKSMulSubOps};

use super::helpers::{TestContext, TestMulBackend as Backend, TestScalar, TestVector, assert_ct_meta};
use poulpy_hal::api::NegacyclicFFT;

fn alloc_scratch<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) -> ScratchOwned<BE> {
    let ct_infos = ctx.ct_infos();
    let tsk_infos = ctx.params.tsk_layout();
    let ct_bytes = ctx.module.ckks_mul_sub_ct_tmp_bytes(&ct_infos, &tsk_infos);
    let pt_bytes = ctx.module.ckks_mul_sub_pt_vec_tmp_bytes(&ct_infos, &ct_infos, &ctx.meta_pt());
    let const_bytes = ctx
        .module
        .ckks_mul_sub_pt_const_tmp_bytes(&ct_infos, &ct_infos, &ctx.meta_pt());
    let bytes = ct_bytes.max(pt_bytes).max(const_bytes);
    ScratchOwned::<BE>::alloc(ctx.scratch_size.max(bytes))
}

fn scaled<F: TestScalar>(v: &[F], scale: F) -> Vec<F> {
    v.iter().copied().map(|x| x * scale).collect()
}

fn cmul<F: TestScalar>(a_re: &[F], a_im: &[F], b_re: &[F], b_im: &[F]) -> (Vec<F>, Vec<F>) {
    let m = a_re.len();
    let mut re = Vec::with_capacity(m);
    let mut im = Vec::with_capacity(m);
    for i in 0..m {
        re.push(a_re[i] * b_re[i] - a_im[i] * b_im[i]);
        im.push(a_re[i] * b_im[i] + a_im[i] * b_re[i]);
    }
    (re, im)
}

fn cmul_scalar<F: TestScalar>(a_re: &[F], a_im: &[F], c_re: F, c_im: F) -> (Vec<F>, Vec<F>) {
    let m = a_re.len();
    let mut re = Vec::with_capacity(m);
    let mut im = Vec::with_capacity(m);
    for i in 0..m {
        re.push(a_re[i] * c_re - a_im[i] * c_im);
        im.push(a_re[i] * c_im + a_im[i] * c_re);
    }
    (re, im)
}

pub fn test_mul_sub_ct_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let half = F::from_f64(0.5).unwrap();
    let dst_re = scaled(&ctx.re1, half);
    let dst_im = scaled(&ctx.im1, half);
    let a_re = scaled(&ctx.re2, half);
    let a_im = scaled(&ctx.im2, half);
    let b_re = scaled(&ctx.re1, half);
    let b_im = scaled(&ctx.im2, half);

    let (prod_re, prod_im) = cmul(&a_re, &a_im, &b_re, &b_im);
    let want_re: Vec<F> = (0..dst_re.len()).map(|i| dst_re[i] - prod_re[i]).collect();
    let want_im: Vec<F> = (0..dst_im.len()).map(|i| dst_im[i] - prod_im[i]).collect();

    let mut dst = ctx.encrypt(ctx.max_k(), &dst_re, &dst_im, &mut scratch.borrow());
    let a = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let b = ctx.encrypt(ctx.max_k(), &b_re, &b_im, &mut scratch.borrow());
    ctx.module
        .ckks_mul_sub_ct_into(&mut dst, &a, &b, ctx.tsk(), &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision("mul_sub_ct_aligned", &dst, &want_re, &want_im, &mut scratch.borrow());
}

/// `dst` starts at a lower `log_budget` than the product `a·b`, forcing
/// alignment inside the fused sub.
pub fn test_mul_sub_ct_unaligned_dst<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let half = F::from_f64(0.5).unwrap();
    let dst_re = scaled(&ctx.re1, half);
    let dst_im = scaled(&ctx.im1, half);
    let a_re = scaled(&ctx.re2, half);
    let a_im = scaled(&ctx.im2, half);
    let b_re = scaled(&ctx.re1, half);
    let b_im = scaled(&ctx.im2, half);

    let (prod_re, prod_im) = cmul(&a_re, &a_im, &b_re, &b_im);
    let want_re: Vec<F> = (0..dst_re.len()).map(|i| dst_re[i] - prod_re[i]).collect();
    let want_im: Vec<F> = (0..dst_im.len()).map(|i| dst_im[i] - prod_im[i]).collect();

    let smaller_k = ctx.max_k() - ctx.base2k().as_usize() + 1;
    let mut dst = ctx.encrypt(smaller_k, &dst_re, &dst_im, &mut scratch.borrow());
    let a = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let b = ctx.encrypt(ctx.max_k(), &b_re, &b_im, &mut scratch.borrow());
    ctx.module
        .ckks_mul_sub_ct_into(&mut dst, &a, &b, ctx.tsk(), &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision("mul_sub_ct_unaligned_dst", &dst, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_mul_sub_pt_vec_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let half = F::from_f64(0.5).unwrap();
    let dst_re = scaled(&ctx.re1, half);
    let dst_im = scaled(&ctx.im1, half);
    let a_re = scaled(&ctx.re2, half);
    let a_im = scaled(&ctx.im2, half);
    let b_re_raw = scaled(&ctx.re1, half);
    let b_im_raw = scaled(&ctx.im2, half);
    let (b_re, b_im) = ctx.quantized_pt_slots(&b_re_raw, &b_im_raw);

    let (prod_re, prod_im) = cmul(&a_re, &a_im, &b_re, &b_im);
    let want_re: Vec<F> = (0..dst_re.len()).map(|i| dst_re[i] - prod_re[i]).collect();
    let want_im: Vec<F> = (0..dst_im.len()).map(|i| dst_im[i] - prod_im[i]).collect();

    let mut dst = ctx.encrypt(ctx.max_k(), &dst_re, &dst_im, &mut scratch.borrow());
    let a = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let pt = ctx.encode_pt(&b_re_raw, &b_im_raw);
    ctx.module
        .ckks_mul_sub_pt_vec_into(&mut dst, &a, &pt, &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision("mul_sub_pt_vec_aligned", &dst, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_mul_sub_pt_vec_into_delta_log_delta<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let half = F::from_f64(0.5).unwrap();
    let pt_prec = ctx.meta_pt();
    let dst_re = scaled(&ctx.re1, half);
    let dst_im = scaled(&ctx.im1, half);
    let (a_re, a_im) = ctx.quantized_vector(TestVector::First, ctx.meta().log_delta);
    let a_re = scaled(&a_re, half);
    let a_im = scaled(&a_im, half);
    let b_re_raw = scaled(&ctx.re2, half);
    let b_im_raw = scaled(&ctx.im2, half);
    let (b_re, b_im) = ctx.quantized_pt_slots(&b_re_raw, &b_im_raw);

    let (prod_re, prod_im) = cmul(&a_re, &a_im, &b_re, &b_im);
    let want_re: Vec<F> = (0..dst_re.len()).map(|i| dst_re[i] - prod_re[i]).collect();
    let want_im: Vec<F> = (0..dst_im.len()).map(|i| dst_im[i] - prod_im[i]).collect();

    let mut dst = ctx.encrypt(ctx.max_k(), &dst_re, &dst_im, &mut scratch.borrow());
    let a = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let pt = ctx.encode_pt_with_prec(&b_re_raw, &b_im_raw, pt_prec);
    ctx.module
        .ckks_mul_sub_pt_vec_into(&mut dst, &a, &pt, &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision_at_log_delta(
        "mul_sub_pt_vec_into_delta_log_delta",
        &dst,
        &want_re,
        &want_im,
        pt_prec.log_delta(),
        &mut scratch.borrow(),
    );
}

pub fn test_mul_sub_pt_const_into_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let half = F::from_f64(0.5).unwrap();
    let dst_re = scaled(&ctx.re1, half);
    let dst_im = scaled(&ctx.im1, half);
    let a_re = scaled(&ctx.re2, half);
    let a_im = scaled(&ctx.im2, half);

    let (c_re_f64, _) = ctx.sample_mul_const();
    let (c_re, c_im) = ctx.quantized_const_pt(c_re_f64, 0.0);
    let (prod_re, prod_im) = cmul_scalar(&a_re, &a_im, c_re, c_im);
    let want_re: Vec<F> = (0..dst_re.len()).map(|i| dst_re[i] - prod_re[i]).collect();
    let want_im: Vec<F> = (0..dst_im.len()).map(|i| dst_im[i] - prod_im[i]).collect();

    let mut dst = ctx.encrypt(ctx.max_k(), &dst_re, &dst_im, &mut scratch.borrow());
    let a = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let cst = ctx.const_full(Some(c_re_f64), None, ctx.meta_pt());
    ctx.module
        .ckks_mul_sub_pt_const_into(&mut dst, &a, &cst, 0, &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision(
        "mul_sub_pt_const_into_aligned",
        &dst,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}

pub fn test_mul_sub_pt_const_zero_preserves_dst_meta<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(
    ctx: &TestContext<BE, F, E>,
) {
    let mut scratch = alloc_scratch(ctx);
    let half = F::from_f64(0.5).unwrap();
    let dst_re = scaled(&ctx.re1, half);
    let dst_im = scaled(&ctx.im1, half);
    let a_re = scaled(&ctx.re2, half);
    let a_im = scaled(&ctx.im2, half);

    let mut dst = ctx.encrypt(ctx.max_k(), &dst_re, &dst_im, &mut scratch.borrow());
    let a = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let dst_meta = dst.meta();
    let zero_prec = CKKSMeta {
        log_delta: 0,
        log_budget: ctx.meta_pt().log_budget,
    };
    let cst = ctx.const_full(None, None, zero_prec);
    ctx.module
        .ckks_mul_sub_pt_const_into(&mut dst, &a, &cst, 0, &mut scratch.borrow())
        .unwrap();

    assert_ct_meta("mul_sub_pt_const_zero", &dst, dst_meta.log_delta, dst_meta.log_budget);
    ctx.assert_decrypt_precision("mul_sub_pt_const_zero", &dst, &dst_re, &dst_im, &mut scratch.borrow());
}
