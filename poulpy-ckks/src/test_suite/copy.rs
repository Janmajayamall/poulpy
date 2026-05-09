//! Copy tests.
//!
//! # Test inventory
//!
//! | Function | Path exercised |
//! |----------|----------------|
//! | [`test_copy_aligned`] | CKKS copy into an equally-sized output |
//! | [`test_copy_smaller_output`] | CKKS copy into a smaller output, forcing the left-shift path |

use crate::{CKKSInfos, leveled::api::CKKSCopyOps};

use super::helpers::{TestContext, TestCopyBackend as Backend, TestScalar, assert_unary_output_meta};
use anyhow::Result;
use poulpy_hal::api::NegacyclicFFT;
use poulpy_hal::api::ScratchOwnedBorrow;

pub fn test_copy_aligned<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) -> Result<()> {
    let mut scratch = ctx.alloc_scratch();
    let ct = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let mut ct_res = ctx.alloc_ct(ctx.max_k());

    ctx.module.ckks_copy(&mut ct_res, &ct, &mut scratch.borrow())?;

    assert_unary_output_meta("copy", &ct_res, &ct);
    ctx.assert_decrypt_precision("copy", &ct_res, &ctx.re1, &ctx.im1, &mut scratch.borrow());
    Ok(())
}

pub fn test_copy_smaller_output<BE: Backend, F: TestScalar, E: NegacyclicFFT<F>>(ctx: &TestContext<BE, F, E>) -> Result<()> {
    let mut scratch = ctx.alloc_scratch();
    let ct = ctx.encrypt(ctx.max_k(), &ctx.re1, &ctx.im1, &mut scratch.borrow());
    let mut ct_res = ctx.alloc_ct(ct.effective_k() - 1);

    ctx.module.ckks_copy(&mut ct_res, &ct, &mut scratch.borrow())?;

    assert_unary_output_meta("copy smaller_output", &ct_res, &ct);
    ctx.assert_decrypt_precision("copy smaller_output", &ct_res, &ctx.re1, &ctx.im1, &mut scratch.borrow());
    Ok(())
}
