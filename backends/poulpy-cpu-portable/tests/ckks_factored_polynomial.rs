#![cfg(feature = "enable-ckks")]

use poulpy_ckks::{
    CKKSCtBounds, CKKSLayout, CoeffsMeta, GLWEPlaintextMeta, ReplicatedLayout, Scale, SlotLayout, SlotsKind,
    api::{CKKSAddOps, CKKSCopyOps, CKKSFactoredPolynomialEvaluationOps, CKKSMulOps},
    layouts::{CKKSCiphertextAlloc, CKKSMaskedConstants},
    polynomial::{EncodedFactoredPolynomial, Factor, FactorStep, FactorUpdate, FactoredPolynomial},
    test_suite::parity::{
        helpers::{fixture_ciphertext, snapshot, with_scratch},
        keys::{key_layout, prepared_tensor_key},
    },
};
use poulpy_core::layouts::{GLWELayout, GLWEToBackendRef, IntPolyInfos};
use poulpy_cpu_portable::FFT64Portable;
use poulpy_hal::layouts::{Module, Standard};

type BE = FFT64Portable;

fn check<P>(module: &Module<BE>, slots: SlotLayout, encoded: EncodedFactoredPolynomial<P>) -> anyhow::Result<()>
where
    P: GLWEToBackendRef<BE, Standard> + CKKSCtBounds + IntPolyInfos + CKKSMaskedConstants<BE, Standard>,
{
    let layout = CKKSLayout {
        glwe_layout: GLWELayout {
            n: 64u32.into(),
            base2k: 16u32.into(),
            k: 193u32.into(),
            rank: 1u32.into(),
        },
        meta: GLWEPlaintextMeta {
            scale: Scale::Log(16),
            slots: SlotsKind::Real,
            slot_layout: slots,
            ..Default::default()
        },
    };
    let key_shape = key_layout(64, 16, 257, 1, 1, 1);
    let key = prepared_tensor_key::<BE, Standard>(module, &key_shape, 98);
    let input = fixture_ciphertext::<BE, Standard>(module, &layout, 37);
    let mut actual = module.ckks_ciphertext_alloc_from_infos(&layout);
    let size = CKKSFactoredPolynomialEvaluationOps::<BE, Standard>::ckks_eval_poly_factored_tmp_bytes(
        module, &actual, &input, &encoded, &key_shape,
    );
    with_scratch::<BE, _>(size, |s| {
        module.ckks_eval_poly_factored(&mut actual, &input, &encoded, &key, s)
    })?;

    let mut expected = module.ckks_ciphertext_alloc_from_infos(&layout);
    let mut square = module.ckks_ciphertext_alloc_from_infos(&layout);
    let mut a = module.ckks_ciphertext_alloc_from_infos(&layout);
    let mut b = module.ckks_ciphertext_alloc_from_infos(&layout);
    let coeffs = encoded.coefficients();
    with_scratch::<BE, _>(size, |s| -> anyhow::Result<()> {
        module.ckks_square_into(&mut square, &input, &key, s)?;
        module.ckks_copy(&mut b, &square, s)?;
        if let Some(c) = coeffs.masked_constant(0) {
            module.ckks_mul_pt_vec_into(&mut a, &square, c, s)?;
            module.ckks_add_pt_vec_assign(&mut a, coeffs.masked_constant(1).unwrap(), s)?;
            module.ckks_add_pt_vec_assign(&mut b, coeffs.masked_constant(2).unwrap(), s)?;
        } else {
            module.ckks_mul_pt_const_into(&mut a, &square, coeffs, 0, s)?;
            module.ckks_add_pt_const_assign(&mut a, 0, coeffs, 1, s)?;
            module.ckks_add_pt_const_assign(&mut b, 0, coeffs, 2, s)?;
        }
        module.ckks_mul_into(&mut expected, &a, &b, &key, s)?;
        Ok(())
    })?;
    assert_eq!(snapshot::<BE, _, Standard>(&actual), snapshot::<BE, _, Standard>(&expected));
    Ok(())
}

#[test]
fn shared_factors_match_direct_arithmetic_on_full_and_replicated_slots() -> anyhow::Result<()> {
    use Factor::{Product, Square, Value, WeightedSum};
    use FactorUpdate::AddConstant;
    // (0.75 x² + 1)(x² − 0.25).
    let p = FactoredPolynomial::new(
        vec![0.75, 1.0, -0.25],
        vec![
            FactorStep::new(Square(0), []),
            FactorStep::new(WeightedSum(vec![(1, 0)]), [AddConstant(1)]),
            FactorStep::new(Value(1), [AddConstant(2)]),
            FactorStep::new(Product(2, 3), []),
        ],
    )?;
    assert_eq!(p.required_budget_bits(16, 12), 44);
    let module = Module::<BE>::new(64);
    let meta = CoeffsMeta::from_delta_budget(12, 4);
    let full = with_scratch::<BE, _>(1 << 16, |s| p.clone().encode::<BE, Standard>(&module, 16u32.into(), meta, s))?;
    check(&module, SlotLayout::Full, full)?;
    let layout = ReplicatedLayout::new(2, 1, &[1, -1, 0, 1])?;
    let masked = with_scratch::<BE, _>(1 << 16, |s| {
        p.encode_replicated::<BE, Standard>(&module, 16u32.into(), meta, layout, s)
    })?;
    check(&module, SlotLayout::Replicated(layout), masked)
}
