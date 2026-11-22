//! A straight-line representation of a real polynomial with shared intermediate values.
use anyhow::{Result, ensure};

use crate::{
    CKKSModuleInfos, CoeffsMeta, ReplicatedLayout,
    api::CKKSEncodingOps,
    layouts::{
        CKKSPlaintextAlloc, CKKSPlaintextOwned, MaskedCoeffs, eval_mod::encode_scalar_constants, masked::encode_masked_constants,
    },
};
use poulpy_core::layouts::{Base2K, GetDegree};
use poulpy_hal::layouts::{Backend, Module, Ring, ScratchArena};

/// Value 0 is the input. Step `i` (counting from 1) produces value `i`.
#[derive(Clone, Debug)]
pub enum Factor {
    /// Start with an earlier value.
    Value(usize),
    Square(usize),
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
                Factor::Value(v) | Factor::Square(v) => reference(*v)?,
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
                    FactorUpdate::MulPow2(bits) => ensure!(bits <= i32::MAX as u32, "power of two exceeds exponent range"),
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
        self.steps
            .iter()
            .any(|s| matches!(s.factor, Factor::Square(_) | Factor::Product(_, _)))
    }

    pub(crate) fn coefficient_count(&self) -> usize {
        self.constants.len()
    }

    /// Multiplicative levels on the longest dependency chain, including coefficient products.
    pub fn eval_depth(&self) -> usize {
        self.required_budget_bits(1, 1)
    }

    /// Minimum input budget for all factors at the given log scales and natural output widths.
    /// A square or ciphertext product spends the input scale, and a weighted sum spends the coefficient scale once.
    pub fn required_budget_bits(&self, input_log_delta: usize, coeff_log_delta: usize) -> usize {
        let mut used = vec![0usize];
        for step in &self.steps {
            let mut cost = match &step.factor {
                Factor::Value(v) => used[*v],
                Factor::Square(v) => used[*v].saturating_add(input_log_delta),
                Factor::Product(a, b) => used[*a].max(used[*b]).saturating_add(input_log_delta),
                Factor::WeightedSum(terms) => terms
                    .iter()
                    .map(|&(v, _)| used[v])
                    .max()
                    .unwrap()
                    .saturating_add(coeff_log_delta),
            };
            for update in &step.updates {
                if let FactorUpdate::Add(v) = update {
                    cost = cost.max(used[*v]);
                }
            }
            used.push(cost);
        }
        used.into_iter().max().unwrap()
    }

    /// Evaluates the same schedule on a scalar for fitting and diagnostics.
    pub fn evaluate_scalar(&self, x: f64) -> f64 {
        let mut values = vec![x];
        for step in &self.steps {
            let mut result = match &step.factor {
                Factor::Value(v) => values[*v],
                Factor::Square(v) => values[*v] * values[*v],
                Factor::Product(a, b) => values[*a] * values[*b],
                Factor::WeightedSum(terms) => terms.iter().map(|&(v, c)| values[v] * self.constants[c]).sum(),
            };
            for &update in &step.updates {
                match update {
                    FactorUpdate::Add(v) => result += values[v],
                    FactorUpdate::AddConstant(c) => result += self.constants[c],
                    FactorUpdate::MulPow2(bits) => result *= 2f64.powi(bits as i32),
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
