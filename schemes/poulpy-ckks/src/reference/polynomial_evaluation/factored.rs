use crate::{
    CKKSCtBounds, CKKSInfos, CKKSResult as Result, SetCKKSInfos, SlotLayout,
    api::{CKKSAddOps, CKKSCopyOps, CKKSFactoredPolynomialEvaluationOps, CKKSMulAddOps, CKKSMulOps, CKKSPow2Ops},
    ckks_ensure_operand,
    layouts::{
        CKKSCiphertextViewMut, CKKSCiphertextViewRef, CKKSMaskedConstants, ScratchArenaTakeCKKSCiphertext,
        validation::{ensure_masked_constants, ensure_scratch},
    },
    oep::derived::polynomial_evaluation::eval_baby_linear_combination_tmp_bytes,
    polynomial::{EncodedFactoredPolynomial, Factor, FactorStep, FactorUpdate},
};
use poulpy_core::{
    GLWETensoring,
    layouts::{
        GGLWEInfos, GLWEInfos, GLWEPlaintextInfos, GLWEToBackendMut, GLWEToBackendRef, GetTensorKey, IntPolyInfos, LWEInfos, SetK,
    },
    oep::GLWEMulConstImpl,
};
use poulpy_hal::layouts::{Backend, Module, Ring, ScratchArena};

use super::{const_add_assign, const_mul_add_into, const_mul_into, full_baby_linear_combination};

const OP: &str = "ckks_eval_poly_factored";

fn buffer_bytes<BE: Backend>(src: &impl CKKSCtBounds) -> usize {
    BE::scratch_aligned(BE::vec_znx_bytes_of(src.n().into(), (src.rank() + 1).into(), src.size()))
}

impl<BE: Backend + GLWEMulConstImpl<Rg>, Rg: Ring> CKKSFactoredPolynomialEvaluationOps<BE, Rg> for Module<BE>
where
    Self: CKKSAddOps<BE, Rg>
        + CKKSMulOps<BE, Rg>
        + CKKSMulAddOps<BE, Rg>
        + CKKSCopyOps<BE, Rg>
        + CKKSPow2Ops<BE, Rg>
        + GLWETensoring<BE, Rg>,
{
    fn ckks_eval_poly_factored_tmp_bytes<R, S, P, T>(
        &self,
        dst: &R,
        src: &S,
        poly: &EncodedFactoredPolynomial<P>,
        key: &T,
    ) -> usize
    where
        R: CKKSCtBounds,
        S: CKKSCtBounds,
        P: CKKSCtBounds,
        T: GGLWEInfos,
    {
        let c = &poly.coefficients;
        let width = src.max_size().max(dst.max_size());
        let mut operation = self
            .ckks_mul_tmp_bytes(src, src, src, key)
            .max(self.ckks_mul_tmp_bytes(dst, src, src, key))
            .max(self.ckks_square_tmp_bytes(src, src, key))
            .max(self.ckks_square_tmp_bytes(dst, src, key))
            .max(self.ckks_copy_tmp_bytes(src, src))
            .max(self.ckks_copy_tmp_bytes(dst, src))
            .max(self.ckks_add_tmp_bytes(width))
            .max(self.ckks_mul_pow2_tmp_bytes(width))
            .max(self.ckks_add_pt_const_tmp_bytes(width))
            .max(self.ckks_mul_pt_const_tmp_bytes(src, src, c))
            .max(self.ckks_mul_pt_const_tmp_bytes(dst, src, c))
            .max(self.ckks_mul_add_pt_const_tmp_bytes(src, src, c))
            .max(self.ckks_mul_add_pt_const_tmp_bytes(dst, src, c))
            .max(eval_baby_linear_combination_tmp_bytes::<BE, Rg>(self, width, c.size()));
        if matches!(src.slot_layout(), SlotLayout::Replicated(_)) {
            operation = operation
                .max(self.ckks_add_pt_vec_tmp_bytes(width))
                .max(self.ckks_mul_pt_vec_tmp_bytes(src, src, c))
                .max(self.ckks_mul_pt_vec_tmp_bytes(dst, src, c))
                .max(self.ckks_mul_add_pt_vec_tmp_bytes(src, src, c))
                .max(self.ckks_mul_add_pt_vec_tmp_bytes(dst, src, c));
        }
        poly.plan.temporaries * buffer_bytes::<BE>(src) + operation
    }

    fn ckks_eval_poly_factored<R, S, P, H>(
        &self,
        dst: &mut R,
        src: &S,
        poly: &EncodedFactoredPolynomial<P>,
        key: &H,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        R: GLWEToBackendMut<BE, Rg> + GLWEToBackendRef<BE, Rg> + CKKSCtBounds + SetCKKSInfos,
        S: GLWEToBackendRef<BE, Rg> + CKKSCtBounds,
        P: GLWEToBackendRef<BE, Rg> + CKKSCtBounds + IntPolyInfos + CKKSMaskedConstants<BE, Rg>,
        H: GetTensorKey<BE, Rg>,
    {
        crate::ensure_operand_degrees(OP, self, [src.n(), dst.n()])?;
        ckks_ensure_operand!(
            OP,
            src.n() == dst.n() && src.rank() == dst.rank(),
            "input and output shapes differ"
        );
        ckks_ensure_operand!(
            OP,
            poly.coefficients.n().as_usize() >= poly.plan.coefficient_count(),
            "encoded scalar table is too short"
        );
        if poly.plan.coefficient_count() > 0 {
            ensure_masked_constants::<BE, Rg, _>(OP, src.slot_layout(), [&poly.coefficients])?;
        }
        crate::error::checked_log_budget_sub(
            OP,
            src.log_budget(),
            poly.plan.required_budget_bits(src.log_delta(), poly.coefficients.log_delta()),
        )?;
        let layout = src.glwe_layout();
        ensure_scratch(OP, scratch, poly.plan.temporaries * buffer_bytes::<BE>(src))?;
        let (mut free, mut arena) =
            scratch
                .borrow()
                .take_ckks_ciphertext_slice_scratch::<_, Rg>(poly.plan.temporaries, &layout, src.meta());
        // Value 0 aliases the caller's input. Released slots keep no borrow on a recycled ciphertext.
        let mut values: Vec<Option<CKKSCiphertextViewMut<'_, BE, Rg>>> = vec![None];
        for (i, step) in poly.plan.steps.iter().enumerate() {
            let get = |v: usize| {
                if v == 0 {
                    CKKSCiphertextViewRef::of(src)
                } else {
                    values[v]
                        .as_ref()
                        .expect("live input in checked factor plan")
                        .to_backend_view_ref()
                }
            };
            if i + 1 == poly.plan.steps.len() {
                return evaluate_step(self, dst, step, get, &poly.coefficients, key, &mut arena);
            }
            let mut out = free.pop().expect("factor temporary count was precomputed");
            out.set_k(layout.k);
            out.set_meta(src.meta());
            evaluate_step(self, &mut out, step, get, &poly.coefficients, key, &mut arena)?;
            values.push(Some(out));
            for &v in &poly.plan.release_after[i] {
                free.push(values[v].take().expect("factor released once"));
            }
        }
        unreachable!("a checked plan includes an output")
    }
}

fn evaluate_step<'v, BE, Rg, R, P, H>(
    module: &Module<BE>,
    out: &mut R,
    step: &FactorStep,
    get: impl Fn(usize) -> CKKSCiphertextViewRef<'v, BE, Rg>,
    constants: &P,
    key: &H,
    scratch: &mut ScratchArena<'_, BE>,
) -> Result<()>
where
    BE: Backend + GLWEMulConstImpl<Rg> + 'v,
    Rg: Ring,
    Module<BE>: CKKSAddOps<BE, Rg>
        + CKKSMulOps<BE, Rg>
        + CKKSMulAddOps<BE, Rg>
        + CKKSCopyOps<BE, Rg>
        + CKKSPow2Ops<BE, Rg>
        + GLWETensoring<BE, Rg>,
    R: GLWEToBackendMut<BE, Rg> + GLWEToBackendRef<BE, Rg> + CKKSCtBounds + SetCKKSInfos,
    P: GLWEToBackendRef<BE, Rg> + CKKSCtBounds + IntPolyInfos + CKKSMaskedConstants<BE, Rg>,
    H: GetTensorKey<BE, Rg>,
{
    match &step.factor {
        Factor::Value(v) => module.ckks_copy(out, &get(*v), scratch)?,
        Factor::Square(v) => module.ckks_square_into(out, &get(*v), key, scratch)?,
        Factor::GuardedSquare { value, guard_bits } => {
            let src = get(*value);
            let bits = *guard_bits as usize;
            ckks_ensure_operand!(
                OP,
                src.slot_layout() == SlotLayout::Full && bits <= src.log_delta(),
                "guarded square needs full packing and at most one input scale of guard bits"
            );
            crate::reference::mul::square_wide_into(OP, module, out, &src, key, bits, scratch)?;
        }
        Factor::Product(a, b) => module.ckks_mul_into(out, &get(*a), &get(*b), key, scratch)?,
        Factor::WeightedSum(weights) => {
            let operands: Vec<_> = weights.iter().map(|&(v, _)| get(v)).collect();
            let terms: Vec<_> = operands.iter().zip(weights).map(|(ct, &(_, idx))| (ct, idx)).collect();
            let (&(seed, idx), rest) = terms.split_last().expect("weighted sum is nonempty");
            // A narrower destination must use the ordinary product's capacity-aware rounding.
            let fused = if seed.slot_layout() == SlotLayout::Full
                && constants.base2k() == out.base2k()
                && operands.iter().all(|ct| ct.k() <= out.k())
            {
                let width = operands.iter().map(|ct| ct.size()).max().unwrap().max(out.size());
                ensure_scratch(
                    OP,
                    scratch,
                    eval_baby_linear_combination_tmp_bytes::<BE, Rg>(module, width, constants.size()),
                )?;
                full_baby_linear_combination(OP, module, out, &terms, seed, constants, scratch)
            } else {
                false
            };
            if !fused {
                const_mul_into(OP, module, out, seed, constants, idx, scratch)?;
                for &(ct, idx) in rest {
                    const_mul_add_into(OP, module, out, ct, constants, idx, scratch)?;
                }
            }
        }
    }
    for &update in &step.updates {
        match update {
            FactorUpdate::Add(v) => module.ckks_add_assign(out, &get(v), scratch)?,
            FactorUpdate::AddConstant(c) => const_add_assign(OP, module, out, constants, c, scratch)?,
            FactorUpdate::MulPow2(bits) => module.ckks_mul_pow2_assign(bits as usize, out, scratch)?,
            FactorUpdate::MulPow2AtFixedWidth(bits) => {
                let scale = crate::error::checked_log_budget_sub(OP, out.log_delta(), bits as usize)?;
                let mut meta = out.meta();
                meta.scale = crate::Scale::Log(scale);
                out.set_meta(meta);
            }
            FactorUpdate::DropPrecision(bits) => {
                ckks_ensure_operand!(
                    OP,
                    out.slot_layout() == SlotLayout::Full,
                    "dropping precision needs full packing"
                );
                let scale = crate::error::checked_log_budget_sub(OP, out.log_delta(), bits as usize)?;
                out.set_log_delta(scale);
            }
        }
    }
    Ok(())
}
