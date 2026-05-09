//! Tests for `CKKSAffineOps` — scalar `offset + scale * ct`.
//!
//! # Test inventory
//!
//! | Function | Path exercised |
//! |----------|----------------|
//! | [`test_affine_pt_const_into_aligned`] | scalar affine op with aligned output |
//! | [`test_affine_pt_const_zero_bias_matches_mul`] | zero offset preserves multiply result metadata |
//! | [`test_affine_pt_const_assign_aligned`] | assign variant of scalar affine op |
//! | [`test_affine_pt_vec_into_aligned`] | full-vector affine op into a fresh destination |
//! | [`test_affine_pt_vec_assign_aligned`] | full-vector affine assign in-place |

use poulpy_hal::{
    api::{NegacyclicFFT, ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::ScratchOwned,
};

use crate::{
    CKKSInfos, CKKSMeta,
    layouts::{CKKSModuleAlloc, CKKSPlaintext, CKKSPlaintextVecHostCodec},
    leveled::api::CKKSAffineOps,
};

use super::helpers::{TestContext, TestMulBackend as Backend, TestScalar, assert_ct_meta};

fn alloc_scratch<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) -> ScratchOwned<BE> {
    let ct_infos = ctx.ct_infos();
    let bytes = ctx
        .module
        .ckks_affine_pt_const_tmp_bytes(&ct_infos, &ct_infos, &ctx.meta_pt());
    ScratchOwned::<BE>::alloc(ctx.scratch_size.max(bytes))
}

fn scaled<F: TestScalar>(v: &[F], scale: F) -> Vec<F> {
    v.iter().copied().map(|x| x * scale).collect()
}

fn encode_affine_const<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(
    ctx: &TestContext<BE, F, E>,
    offset: F,
    scale: F,
    prec: CKKSMeta,
) -> CKKSPlaintext<Vec<u8>> {
    let mut pt = ctx.host_module.ckks_pt_coeffs_alloc(2, ctx.base2k(), prec);
    pt.encode_host_floats(&[offset, scale]).unwrap();
    pt
}

pub fn test_affine_pt_const_into_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let half = F::from_f64(0.5).unwrap();
    let a_re = scaled(&ctx.re1, half);
    let a_im = vec![F::zero(); a_re.len()];

    let (offset_f64, _) = ctx.sample_add_sub_const();
    let (scale_f64, _) = ctx.sample_mul_const();
    let offset = ctx.quantized_const_pt(offset_f64, 0.0).0;
    let scale = ctx.quantized_const_pt(scale_f64, 0.0).0;
    let want_re: Vec<F> = a_re.iter().map(|x| *x * scale + offset).collect();
    let want_im: Vec<F> = vec![F::zero(); a_re.len()];

    let a = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let affine_const = encode_affine_const(ctx, offset, scale, ctx.meta_pt());
    let mut dst = ctx.alloc_ct(ctx.max_k());

    ctx.module
        .ckks_affine_pt_const_into(&mut dst, &a, &affine_const, 0, 1, &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision(
        "affine_pt_const_into_aligned",
        &dst,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}

pub fn test_affine_pt_const_zero_bias_matches_mul<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let half = F::from_f64(0.5).unwrap();
    let a_re = scaled(&ctx.re2, half);
    let a_im = vec![F::zero(); a_re.len()];

    let (scale_f64, _) = ctx.sample_mul_const();
    let scale = ctx.quantized_const_pt(scale_f64, 0.0).0;
    let want_re: Vec<F> = a_re.iter().map(|x| *x * scale).collect();
    let want_im: Vec<F> = vec![F::zero(); a_re.len()];

    let a = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let affine_const = encode_affine_const(ctx, F::zero(), scale, ctx.meta_pt());
    let mut dst = ctx.alloc_ct(ctx.max_k());

    ctx.module
        .ckks_affine_pt_const_into(&mut dst, &a, &affine_const, 0, 1, &mut scratch.borrow())
        .unwrap();

    assert_ct_meta(
        "affine_pt_const_zero_bias",
        &dst,
        a.log_delta(),
        a.log_budget() - ctx.meta_pt().log_delta,
    );
    ctx.assert_decrypt_precision("affine_pt_const_zero_bias", &dst, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_affine_pt_const_assign_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let mut scratch = alloc_scratch(ctx);
    let half = F::from_f64(0.5).unwrap();
    let a_re = scaled(&ctx.re1, half);
    let a_im = vec![F::zero(); a_re.len()];

    let (offset_f64, _) = ctx.sample_add_sub_const();
    let (scale_f64, _) = ctx.sample_mul_const();
    let offset = ctx.quantized_const_pt(offset_f64, 0.0).0;
    let scale = ctx.quantized_const_pt(scale_f64, 0.0).0;
    let want_re: Vec<F> = a_re.iter().map(|x| *x * scale + offset).collect();
    let want_im: Vec<F> = vec![F::zero(); a_re.len()];

    let mut ct = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let affine_const = encode_affine_const(ctx, offset, scale, ctx.meta_pt());

    ctx.module
        .ckks_affine_pt_const_assign(&mut ct, &affine_const, 0, 1, &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision(
        "affine_pt_const_assign_aligned",
        &ct,
        &want_re,
        &want_im,
        &mut scratch.borrow(),
    );
}

pub fn test_affine_pt_vec_into_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let ct_infos = ctx.ct_infos();
    let bytes = ctx.module.ckks_affine_pt_vec_tmp_bytes(&ct_infos, &ct_infos, &ctx.meta_pt());
    let mut scratch = ScratchOwned::<BE>::alloc(ctx.scratch_size.max(bytes));

    let half = F::from_f64(0.5).unwrap();
    let a_re = scaled(&ctx.re1, half);
    let a_im = vec![F::zero(); a_re.len()];

    let (offset_f64, _) = ctx.sample_add_sub_const();
    let (scale_f64, _) = ctx.sample_mul_const();
    let offset = ctx.quantized_const_pt(offset_f64, 0.0).0;
    let scale = ctx.quantized_const_pt(scale_f64, 0.0).0;
    let want_re: Vec<F> = a_re.iter().map(|x| *x * scale + offset).collect();
    let want_im: Vec<F> = vec![F::zero(); a_re.len()];

    let a = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let scale_pt = ctx.const_full_rnx(Some(scale_f64), None, ctx.meta_pt());
    let offset_pt = ctx.const_full_rnx(Some(offset_f64), None, ctx.meta_pt());
    let mut dst = ctx.alloc_ct(ctx.max_k());

    ctx.module
        .ckks_affine_pt_vec_into(&mut dst, &a, &scale_pt, &offset_pt, &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision("affine_pt_vec_into_aligned", &dst, &want_re, &want_im, &mut scratch.borrow());
}

pub fn test_affine_pt_vec_assign_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) {
    let ct_infos = ctx.ct_infos();
    let bytes = ctx.module.ckks_affine_pt_vec_tmp_bytes(&ct_infos, &ct_infos, &ctx.meta_pt());
    let mut scratch = ScratchOwned::<BE>::alloc(ctx.scratch_size.max(bytes));

    let half = F::from_f64(0.5).unwrap();
    let a_re = scaled(&ctx.re1, half);
    let a_im = vec![F::zero(); a_re.len()];

    let (offset_f64, _) = ctx.sample_add_sub_const();
    let (scale_f64, _) = ctx.sample_mul_const();
    let offset = ctx.quantized_const_pt(offset_f64, 0.0).0;
    let scale = ctx.quantized_const_pt(scale_f64, 0.0).0;
    let want_re: Vec<F> = a_re.iter().map(|x| *x * scale + offset).collect();
    let want_im: Vec<F> = vec![F::zero(); a_re.len()];

    let mut ct = ctx.encrypt(ctx.max_k(), &a_re, &a_im, &mut scratch.borrow());
    let scale_pt = ctx.const_full_rnx(Some(scale_f64), None, ctx.meta_pt());
    let offset_pt = ctx.const_full_rnx(Some(offset_f64), None, ctx.meta_pt());

    ctx.module
        .ckks_affine_pt_vec_assign(&mut ct, &scale_pt, &offset_pt, &mut scratch.borrow())
        .unwrap();
    ctx.assert_decrypt_precision("affine_pt_vec_assign_aligned", &ct, &want_re, &want_im, &mut scratch.borrow());
}
