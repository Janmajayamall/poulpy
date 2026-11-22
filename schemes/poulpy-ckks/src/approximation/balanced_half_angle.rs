//! Centered coordinates for the degree-56 half-angle fit.
//!
//! In the original fit, write h = (R² + D) S, R = q + r, S = q m + l,
//! with r, D, m and l affine in z,t. Recenter at t = 2.5 and q = 7,
//! then use Z = 4z, T = 4(t - 2.5), Q = 4(q - 7).
//! The four rows below give 4R - Q, √2 m/64, √2 (l + 7m)/16 and 16D
//! in [1,Z,T]. Their product circuit returns √2 h before its last square.
//! Every encoded coefficient fits in the plan's four signed integer bits.
use super::half_angle_cos::{self, COEFF_BITS};
use crate::{
    numerics::CKKSFloat,
    polynomial::{Factor, FactorStep, FactorUpdate, FactoredPolynomial, Polynomial},
};

pub(crate) const LOG_DELTA: usize = 58;
const POWER_GUARD: u32 = 4;
const COORD_SHIFT: u32 = 2;
const PRODUCT_DROP: u32 = 2;
const T_CENTER: f64 = 2.5;
const Q_CENTER: f64 = 7.0;
const AMPLITUDE: f64 = (1 << COORD_SHIFT) as f64;

// Convert a + b z + c t to the centered basis and scale the whole row.
const fn centered([a, b, c]: [f64; 3], gain: f64) -> [f64; 3] {
    [(a + c * T_CENTER) * gain, b / AMPLITUDE * gain, c / AMPLITUDE * gain]
}

const ROWS: [[f64; 3]; 4] = {
    let [r, m, l, d] = half_angle_cos::ROWS;
    let m0 = m[0] + l[3];
    let gain = std::f64::consts::SQRT_2 / 16.0;
    [
        centered([r[0] + Q_CENTER, r[1], r[2]], AMPLITUDE),
        centered([m0, m[1], m[2]], gain / AMPLITUDE),
        centered([l[0] + Q_CENTER * m0, l[1] + Q_CENTER * m[1], l[2] + Q_CENTER * m[2]], gain),
        centered([d[0], d[1], d[2]], AMPLITUDE * AMPLITUDE),
    ]
};

fn constants() -> Vec<f64> {
    let scale = (1u64 << COEFF_BITS) as f64;
    [
        half_angle_cos::Z_OFFSET,
        half_angle_cos::T_OFFSET,
        -T_CENTER,
        -Q_CENTER,
        -std::f64::consts::TAU.ckks_powf(-0.25),
    ]
    .into_iter()
    .chain(ROWS.iter().flatten().copied())
    .map(|v| (v * scale).round() / scale)
    .collect()
}

fn row_start(row: usize) -> usize {
    5 + 3 * row
}

/// The early powers keep four guard bits. Both centered factors finish at scale 2^56.
pub(crate) fn factorization() -> anyhow::Result<FactoredPolynomial> {
    use Factor::{GuardedSquare, Product, Square, Value, WeightedSum};
    use FactorUpdate::{Add, AddConstant, DropPrecision, MulPow2AtFixedWidth};
    let row = |i: usize, extra: Option<usize>| {
        let first = row_start(i);
        let updates = std::iter::once(AddConstant(first)).chain(extra.map(Add)).collect::<Vec<_>>();
        FactorStep::new(WeightedSum(vec![(4, first + 1), (5, first + 2)]), updates)
    };
    // Coordinates are at scale 2^(58 + 4 - 2). Drop to 2^(58 - 2) before multiplying the two factors.
    let drop = POWER_GUARD - COORD_SHIFT + PRODUCT_DROP;
    FactoredPolynomial::new(
        constants(),
        vec![
            FactorStep::new(
                GuardedSquare {
                    value: 0,
                    guard_bits: POWER_GUARD,
                },
                [AddConstant(0)],
            ), // 1: z
            FactorStep::new(Square(1), [AddConstant(1)]), // 2: t
            FactorStep::new(Square(2), [AddConstant(3), MulPow2AtFixedWidth(COORD_SHIFT)]), // 3: Q
            FactorStep::new(Value(1), [MulPow2AtFixedWidth(COORD_SHIFT)]), // 4: Z
            FactorStep::new(Value(2), [AddConstant(2), MulPow2AtFixedWidth(COORD_SHIFT)]), // 5: T
            row(0, Some(3)),                              // 6: R
            row(3, None),                                 // 7: D
            FactorStep::new(Square(6), [Add(7), DropPrecision(drop)]), // 8: octic
            row(1, None),                                 // 9: m
            row(2, None),                                 // 10: l
            FactorStep::new(Product(9, 3), [Add(10), DropPrecision(drop)]), // 11: sextic
            FactorStep::new(Product(8, 11), []),
            FactorStep::new(Square(12), [AddConstant(4)]),
        ],
    )
}

/// Recover the even Chebyshev coefficients from the quantized factor schedule.
pub(crate) fn coefficients() -> anyhow::Result<Vec<f64>> {
    let c = constants();
    // Interpolate in y = T₂(x), where x² = (y + 1)/2 exactly.
    let folded = Polynomial::chebyshev_interpolate(28, -1.0, 1.0, |y| {
        let z = 0.5 * (y + 1.0) + c[0];
        let t = z * z + c[1];
        let q = AMPLITUDE * (t * t + c[3]);
        let row = |i| {
            let j = row_start(i);
            c[j] + c[j + 1] * (AMPLITUDE * z) + c[j + 2] * (AMPLITUDE * (t + c[2]))
        };
        let r = q + row(0);
        let octic = r * r + row(3);
        let sextic = q * row(1) + row(2);
        let product = octic * sextic;
        product * product + c[4]
    })?;
    let mut result = vec![0.0; 57];
    for (out, c) in result.iter_mut().step_by(2).zip(folded.coeffs) {
        *out = c;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::bootstrapping::n16_d35_k685_p19_s2c_ffr_ci;

    #[test]
    fn centered_circuit_matches_the_planned_budget_and_wrap_bounds() -> anyhow::Result<()> {
        let stock = n16_d35_k685_p19_s2c_ffr_ci()?;
        let mut scales = stock.scales();
        scales.c2s_log_delta = 49;
        scales.eval_mod_log_delta = LOG_DELTA;
        scales.eval_mod_coeffs_log_delta = COEFF_BITS;
        let candidate = stock.with_eval_mod_polynomial(16, 52, 2)?.with_scales(scales)?;
        let plan = candidate.plan().eval_mod();
        assert!(half_angle_cos::matches_plan(plan));
        let constants = constants();
        assert!(constants.iter().all(|c| (-8.0..8.0).contains(c)));
        let fit = factorization()?;
        let dense = Polynomial::new(crate::polynomial::Basis::Chebyshev, coefficients()?);
        for i in -64..=64 {
            let x = f64::from(i) / 64.0;
            assert!((fit.evaluate_scalar(x) - dense.evaluate(x)).abs() < 1e-12);
        }
        let precision = fit.precision_profile(LOG_DELTA, COEFF_BITS);
        assert_eq!(fit.eval_depth(), 6);
        assert_eq!((precision.required_bits, precision.output_scale), (352, 56));
        assert_eq!(plan.eval_depth(), 8);
        assert_eq!(plan.consumed_bits(), precision.required_bits + 2 * precision.output_scale);
        assert_eq!(candidate.bootstrap_k(), 1435);
        let (worst, zero, _) = plan.approximation_errors(1.0)?;
        assert!(zero <= 2f64.powi(-19), "zero-wrap error {zero}");
        assert!(
            worst <= stock.plan().eval_mod().approximation_errors(1.0)?.0,
            "all-wrap error {worst}"
        );
        Ok(())
    }
}
