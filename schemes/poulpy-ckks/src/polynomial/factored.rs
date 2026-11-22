//! A straight-line representation of a real polynomial with shared intermediate values.
use anyhow::{Result, ensure};

use crate::{
    CKKSModuleInfos, CoeffsMeta, ReplicatedLayout,
    api::CKKSEncodingOps,
    layouts::{
        CKKSPlaintextAlloc, CKKSPlaintextOwned, MaskedCoeffs, eval_mod::encode_scalar_constants, masked::encode_masked_constants,
    },
    numerics::CKKSFloat,
};
use poulpy_core::layouts::{Base2K, GetDegree};
use poulpy_hal::layouts::{Backend, Module, Ring, ScratchArena};

/// Value 0 is the input. Step `i` (counting from 1) produces value `i`.
#[derive(Clone, Debug)]
pub enum Factor {
    /// Start with an earlier value.
    Value(usize),
    Square(usize),
    /// Square a full-layout value, retaining extra low bits in the output scale.
    GuardedSquare {
        /// Index of the input value to square.
        value: usize,
        /// Extra bits retained beyond the input scale, up to the input log scale.
        guard_bits: u32,
    },
    Product(usize, usize),
    /// A nonempty sum of (value index, scalar coefficient index) pairs.
    WeightedSum(Vec<(usize, usize)>),
}

/// Operations applied in order to the result of a factor.
#[derive(Clone, Copy, Debug)]
pub enum FactorUpdate {
    Add(usize),
    AddConstant(usize),
    MulPow2(u32),
    /// Multiply by a power of two by lowering the scale at fixed torus width.
    MulPow2AtFixedWidth(u32),
    /// Discard low precision bits of a full-layout value, preserving its homomorphic budget.
    DropPrecision(u32),
}

#[derive(Clone, Debug)]
pub struct FactorStep {
    pub factor: Factor,
    pub updates: Vec<FactorUpdate>,
}

impl FactorStep {
    pub fn new(factor: Factor, updates: impl Into<Vec<FactorUpdate>>) -> Self {
        Self {
            factor,
            updates: updates.into(),
        }
    }
}

/// A checked schedule whose last value is the polynomial result.
/// Repeated references share one ciphertext. Each temporary can be recycled after its final use.
#[derive(Clone, Debug)]
pub struct FactoredPolynomial {
    pub(crate) steps: Vec<FactorStep>,
    constants: Vec<f64>,
    pub(crate) release_after: Vec<Vec<usize>>,
    pub(crate) temporaries: usize,
}

impl FactoredPolynomial {
    /// Checks all value and constant references before encoding or encrypted evaluation.
    pub fn new(constants: Vec<f64>, steps: Vec<FactorStep>) -> Result<Self> {
        ensure!(!steps.is_empty(), "a factored polynomial needs an output step");
        ensure!(
            constants.iter().all(|c| c.is_finite()),
            "nonfinite factored polynomial coefficient"
        );
        let mut last_use: Vec<usize> = (0..=steps.len()).map(|v| v.saturating_sub(1)).collect();
        for (i, step) in steps.iter().enumerate() {
            let mut reference = |v: usize| -> Result<()> {
                ensure!(v <= i, "step {} references unavailable value {v}", i + 1);
                last_use[v] = i;
                Ok(())
            };
            let coefficient = |c: usize| -> Result<()> {
                ensure!(c < constants.len(), "step {} references missing coefficient {c}", i + 1);
                Ok(())
            };
            match &step.factor {
                Factor::Value(v) | Factor::Square(v) | Factor::GuardedSquare { value: v, .. } => reference(*v)?,
                Factor::Product(a, b) => {
                    reference(*a)?;
                    reference(*b)?;
                }
                Factor::WeightedSum(terms) => {
                    ensure!(!terms.is_empty(), "empty weighted sum at step {}", i + 1);
                    for &(v, c) in terms {
                        reference(v)?;
                        coefficient(c)?;
                    }
                }
            }
            for &update in &step.updates {
                match update {
                    FactorUpdate::Add(v) => reference(v)?,
                    FactorUpdate::AddConstant(c) => coefficient(c)?,
                    FactorUpdate::MulPow2(bits) | FactorUpdate::MulPow2AtFixedWidth(bits) | FactorUpdate::DropPrecision(bits) => {
                        ensure!(bits <= i32::MAX as u32, "power of two exceeds exponent range")
                    }
                }
            }
        }
        let mut release_after = vec![Vec::new(); steps.len()];
        for v in 1..steps.len() {
            release_after[last_use[v]].push(v);
        }
        let mut live = 0;
        let mut temporaries = 0;
        // The final step writes into the caller's output.
        for released in &release_after[..steps.len() - 1] {
            live += 1;
            temporaries = temporaries.max(live);
            live -= released.len();
        }
        Ok(Self {
            constants,
            steps,
            release_after,
            temporaries,
        })
    }

    pub(crate) fn has_products(&self) -> bool {
        self.steps.iter().any(|s| {
            matches!(
                s.factor,
                Factor::Square(_) | Factor::GuardedSquare { .. } | Factor::Product(_, _)
            )
        })
    }

    pub(crate) fn coefficient_count(&self) -> usize {
        self.constants.len()
    }

    /// Multiplicative levels on the longest dependency chain, including coefficient products.
    pub fn eval_depth(&self) -> usize {
        self.precision_profile(1, 1).depth
    }

    /// Minimum input budget for all factors at the given log scales and natural output widths.
    pub fn required_budget_bits(&self, input_log_delta: usize, coeff_log_delta: usize) -> usize {
        self.precision_profile(input_log_delta, coeff_log_delta).required_bits
    }

    /// Track the natural output scale separately from the maximum budget used anywhere.
    /// At fixed width, lowering the scale gains budget, including on an early input copy.
    pub(crate) fn precision_profile(&self, input_scale: usize, coeff_scale: usize) -> FactorPrecision {
        #[derive(Clone, Copy)]
        struct Value {
            spent: i128,
            scale: usize,
            depth: usize,
        }

        impl Value {
            fn align(self, other: Self) -> Self {
                Self {
                    spent: self.spent.max(other.spent),
                    scale: self.scale.min(other.scale),
                    depth: self.depth.max(other.depth),
                }
            }

            fn product(mut self, bits: usize) -> Self {
                self.spent += bits as i128;
                self.depth += 1;
                self
            }
        }

        let mut values = vec![Value {
            spent: 0,
            scale: input_scale,
            depth: 0,
        }];
        let mut peak = 0;
        let mut depth = 0;
        for step in &self.steps {
            let mut out = match &step.factor {
                Factor::Value(v) => values[*v],
                Factor::Square(v) => values[*v].product(values[*v].scale),
                Factor::GuardedSquare { value, guard_bits } => {
                    let mut out = values[*value].product(values[*value].scale);
                    out.scale = out.scale.saturating_add(*guard_bits as usize);
                    out
                }
                Factor::Product(a, b) => values[*a].align(values[*b]).product(values[*a].scale.max(values[*b].scale)),
                Factor::WeightedSum(terms) => terms
                    .iter()
                    .map(|&(v, _)| values[v])
                    .reduce(Value::align)
                    .unwrap()
                    .product(coeff_scale),
            };
            peak = peak.max(out.spent);
            for &update in &step.updates {
                match update {
                    FactorUpdate::Add(v) => out = out.align(values[v]),
                    FactorUpdate::MulPow2AtFixedWidth(bits) => {
                        out.scale = out.scale.saturating_sub(bits as usize);
                        out.spent -= i128::from(bits);
                    }
                    FactorUpdate::DropPrecision(bits) => out.scale = out.scale.saturating_sub(bits as usize),
                    FactorUpdate::AddConstant(_) | FactorUpdate::MulPow2(_) => {}
                }
                peak = peak.max(out.spent);
            }
            depth = depth.max(out.depth);
            values.push(out);
        }
        FactorPrecision {
            required_bits: usize::try_from(peak).unwrap_or(usize::MAX),
            output_scale: values.last().unwrap().scale,
            depth,
        }
    }

    /// Evaluates the same schedule on a scalar for fitting and diagnostics.
    pub fn evaluate_scalar(&self, x: f64) -> f64 {
        let mut values = vec![x];
        for step in &self.steps {
            let mut result = match &step.factor {
                Factor::Value(v) => values[*v],
                Factor::Square(v) | Factor::GuardedSquare { value: v, .. } => values[*v] * values[*v],
                Factor::Product(a, b) => values[*a] * values[*b],
                Factor::WeightedSum(terms) => terms.iter().map(|&(v, c)| values[v] * self.constants[c]).sum(),
            };
            for &update in &step.updates {
                match update {
                    FactorUpdate::Add(v) => result += values[v],
                    FactorUpdate::AddConstant(c) => result += self.constants[c],
                    FactorUpdate::MulPow2(bits) | FactorUpdate::MulPow2AtFixedWidth(bits) => {
                        result *= 2f64.ckks_powi(bits as i32)
                    }
                    FactorUpdate::DropPrecision(_) => {}
                }
            }
            values.push(result);
        }
        *values.last().unwrap()
    }

    /// Encode the factor coefficients on the destination backend for full packing.
    pub fn encode<BE: Backend, Rg: Ring>(
        self,
        module: &Module<BE>,
        base2k: Base2K,
        coeff_meta: CoeffsMeta,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<EncodedFactoredPolynomial<CKKSPlaintextOwned<BE, Rg>>>
    where
        Module<BE>: CKKSPlaintextAlloc<BE> + CKKSEncodingOps<BE, f64, Rg>,
    {
        ensure!(
            coeff_meta.k.as_usize() > 0 && base2k.as_usize() > 0,
            "zero coefficient width or radix"
        );
        self.encode_with(|values| encode_scalar_constants(module, base2k, coeff_meta, values, scratch))
    }

    /// Encode each coefficient with the matching masked plaintext for a replicated slot layout.
    pub fn encode_replicated<BE: Backend, Rg: Ring>(
        self,
        module: &Module<BE>,
        base2k: Base2K,
        coeff_meta: CoeffsMeta,
        layout: ReplicatedLayout,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<EncodedFactoredPolynomial<MaskedCoeffs<CKKSPlaintextOwned<BE, Rg>>>>
    where
        Module<BE>: CKKSPlaintextAlloc<BE> + CKKSEncodingOps<BE, f64, Rg> + GetDegree + CKKSModuleInfos,
    {
        ensure!(
            coeff_meta.k.as_usize() > 0 && base2k.as_usize() > 0,
            "zero coefficient width or radix"
        );
        self.encode_with(|values| encode_masked_constants(module, base2k, coeff_meta, layout, values, scratch))
    }

    /// Encodes the scalar table once. A coefficient-free schedule receives one unused zero for the plaintext allocation.
    pub fn encode_with<P>(self, encode: impl FnOnce(&[f64]) -> Result<P>) -> Result<EncodedFactoredPolynomial<P>> {
        let coefficients = encode(if self.constants.is_empty() { &[0.0] } else { &self.constants })?;
        Ok(EncodedFactoredPolynomial {
            plan: self,
            coefficients,
        })
    }
}

pub(crate) struct FactorPrecision {
    pub(crate) required_bits: usize,
    pub(crate) output_scale: usize,
    depth: usize,
}

/// A reusable schedule and its backend-encoded scalar table.
pub struct EncodedFactoredPolynomial<P> {
    pub(crate) plan: FactoredPolynomial,
    pub(crate) coefficients: P,
}

impl<P> EncodedFactoredPolynomial<P> {
    pub fn plan(&self) -> &FactoredPolynomial {
        &self.plan
    }

    /// Backend plaintext containing the encoded scalar table.
    pub fn coefficients(&self) -> &P {
        &self.coefficients
    }

    pub fn map_plaintexts<Q>(self, map: impl FnOnce(&P) -> Q) -> EncodedFactoredPolynomial<Q> {
        EncodedFactoredPolynomial {
            plan: self.plan,
            coefficients: map(&self.coefficients),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factors_share_a_square_and_recycle_temporaries() -> Result<()> {
        use Factor::*;
        use FactorUpdate::*;
        // (x² + 1)(x² − 1), followed by 2*(product + x).
        let p = FactoredPolynomial::new(
            vec![1.0, -1.0],
            vec![
                FactorStep::new(Square(0), []),
                FactorStep::new(Value(1), [AddConstant(0)]),
                FactorStep::new(Value(1), [AddConstant(1)]),
                FactorStep::new(Product(2, 3), [Add(0), MulPow2(1)]),
            ],
        )?;
        for x in [-2.0, -0.75, 0.0, 0.25, 3.0] {
            assert_eq!(p.evaluate_scalar(x), 2.0 * (x.powi(4) - 1.0 + x));
        }
        assert_eq!(p.temporaries, 3);
        assert_eq!(p.eval_depth(), 2);
        assert_eq!(p.required_budget_bits(35, 18), 70);
        Ok(())
    }

    #[test]
    fn rejects_forward_references_and_missing_coefficients() {
        assert!(FactoredPolynomial::new(vec![], vec![FactorStep::new(Factor::Square(1), [])]).is_err());
        assert!(FactoredPolynomial::new(vec![1.0], vec![FactorStep::new(Factor::WeightedSum(vec![(0, 1)]), [])]).is_err());
    }
}
