//! Centered degree-56 Han–Ki base for two double-angle steps.
//!
//! Fit the scaled half-angle cosine, (2π)^(-1/8) cos(4πx), in degree 14
//! in x² at x=(wrap + m/8192 - 1/4)/16 for m=-1,0,1 and |wrap|≤15.
//! Residual weights are 2^(-1.5 max(|wrap|-8,0)) / |sin(4πx)|.
//! Adjust the leading coefficient of the fit so its monic octic factor
//! is a squared quartic plus a quadratic. The remaining sextic takes one
//! product. One doubling gives (2π)^(-1/4) cos(8πx).
use crate::{
    layouts::{EvalModPlan, EvalModType},
    numerics::CKKSFloat,
    polynomial::{Factor, FactorStep, FactorUpdate, FactoredPolynomial, Polynomial, SplitStrategy},
};

pub(super) const COEFF_BITS: usize = 48;
pub(super) const Z_OFFSET: f64 = -1.0393581216405625;
pub(super) const T_OFFSET: f64 = 3.1888881758792054;

// Rows in [1, z, t, q], where z=x²+Z_OFFSET, t=z²+T_OFFSET, q=t².
// R is row 0, the sextic is q·row 1 + row 2, and the octic is R² + row 3.
pub(super) const ROWS: [[f64; 4]; 4] = [
    [9.325560552093284, 0.4817162938382348, -6.222621604398572, 1.0],
    [-18.335325005155592, 28.88602830977092, 5.7497547715358115, 0.0],
    [66.09443599733765, -100.23837443798192, -228.61141067592172, 68.74002734954928],
    [-0.45842185972512295, 0.18872635771786356, 0.11233227538082696, 0.0],
];

// The plaintext has four signed integer bits. Restore each row's power of
// two after combining its encoded coefficients.
const ROW_SHIFTS: [usize; 4] = [1, 2, 5, 0];

pub(crate) fn matches_plan(p: &EvalModPlan) -> bool {
    p.eval_mod_type == EvalModType::CosHKEven
        && p.f_mod_interval == 16
        && p.log_msg_ratio == 13
        && p.f_mod_degree == 52
        && p.f_mod_log_interval_reduction == 2
        && p.split_strategy == SplitStrategy::MinDepth
        && p.scaling.is_none()
        && p.f_mod_inv_degree.is_none()
        && p.coeffs_meta.log_delta() == COEFF_BITS
        && matches!(p.f_mod_log_delta, 59 | super::balanced_half_angle::LOG_DELTA)
        && p.coeffs_meta.log_budget() >= 4
}

/// Quantize the fitted factors before expanding them as Chebyshev coefficients.
fn constants() -> Vec<f64> {
    let scale = (1u64 << COEFF_BITS) as f64;
    let quantize = |c: f64| (c * scale).round() / scale;
    let mut out = vec![
        quantize(Z_OFFSET),
        quantize(T_OFFSET),
        quantize(-std::f64::consts::TAU.ckks_powf(-0.25)),
    ];
    for (row, shift) in ROWS.iter().zip(ROW_SHIFTS) {
        out.extend(row.map(|v| quantize(v / (1 << shift) as f64)));
    }
    out
}

/// The finalist's multiplication order, expressed entirely as polynomial data.
pub(crate) fn factorization() -> anyhow::Result<FactoredPolynomial> {
    use Factor::{Product, Square, WeightedSum};
    use FactorUpdate::{Add, AddConstant, MulPow2};
    let row = |i: usize, count: usize, unscaled_term: Option<usize>| {
        let first = 3 + 4 * i;
        let terms = (1..=count).filter(|&j| ROWS[i][j] != 0.0).map(|j| (j, first + j)).collect();
        let mut updates = vec![AddConstant(first), MulPow2(ROW_SHIFTS[i] as u32)];
        if let Some(value) = unscaled_term {
            updates.push(Add(value));
        }
        FactorStep::new(WeightedSum(terms), updates)
    };
    FactoredPolynomial::new(
        constants(),
        vec![
            FactorStep::new(Square(0), [AddConstant(0)]), // 1: z
            FactorStep::new(Square(1), [AddConstant(1)]), // 2: t
            FactorStep::new(Square(2), []),               // 3: q
            row(0, 2, Some(3)),                           // 4: quartic
            row(3, 3, None),                              // 5: octic correction
            FactorStep::new(Square(4), [Add(5)]),         // 6: octic
            row(1, 2, None),                              // 7: sextic multiplier
            row(2, 3, None),                              // 8: sextic correction
            FactorStep::new(Product(3, 7), [Add(8)]),     // 9: sextic
            FactorStep::new(Product(6, 9), []),           // 10: half-angle
            FactorStep::new(Square(10), [MulPow2(1), AddConstant(2)]),
        ],
    )
}

/// Expand the quantized fit for folded BSGS evaluation. In the folded coordinate y=T₂(x) its degree
/// is 28, so interpolation at 29 Chebyshev nodes recovers the polynomial.
pub(crate) fn coefficients() -> anyhow::Result<Vec<f64>> {
    let constants = constants();
    let folded = Polynomial::chebyshev_interpolate(28, -1.0, 1.0, |y| {
        let z = 0.5 * (y + 1.0) + constants[0];
        let t = z * z + constants[1];
        let q = t * t;
        let row = |i: usize| {
            let start = 3 + 4 * i;
            let c = &constants[start..start + 4];
            (c[0] + c[1] * z + c[2] * t + c[3] * q) * (1 << ROW_SHIFTS[i]) as f64
        };
        let quartic = row(0);
        let sextic = q * row(1) + row(2);
        let octic = quartic * quartic + row(3);
        let half_angle = octic * sextic;
        2.0 * half_angle * half_angle + constants[2]
    })?;
    // T_j(T₂(x)) = T_{2j}(x).
    let mut coeffs = vec![0.0; 57];
    for (c, v) in coeffs.iter_mut().step_by(2).zip(folded.coeffs) {
        *c = v;
    }
    Ok(coeffs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::bootstrapping::n16_d35_k685_p19_s2c_ffr_ci;

    #[test]
    fn quantized_degree_56_fit_covers_the_stock_wraps() -> anyhow::Result<()> {
        let stock = n16_d35_k685_p19_s2c_ffr_ci()?;
        let mut scales = stock.scales();
        scales.eval_mod_log_delta = 59;
        scales.eval_mod_coeffs_log_delta = COEFF_BITS;
        let candidate = stock.with_eval_mod_polynomial(16, 52, 2)?.with_scales(scales)?;
        let plan = candidate.plan().eval_mod();
        assert!(matches_plan(plan));
        assert_eq!(coefficients()?.len(), 57);
        assert_eq!(plan.eval_depth(), 8);
        let (worst, zero, _) = plan.approximation_errors(1.0)?;
        let stock_worst = stock.plan().eval_mod().approximation_errors(1.0)?.0;
        assert!(zero <= 2f64.powi(-19), "zero-wrap error {zero}");
        assert!(worst <= stock_worst, "all-wrap error {worst} exceeds {stock_worst}");
        Ok(())
    }
}
