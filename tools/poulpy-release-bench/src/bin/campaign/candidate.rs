//! Campaign 1 Han–Ki finalists. The default design is the phase-one finalist.
use anyhow::Result;
use poulpy_ckks::presets::bootstrapping::{BootstrappingPreset, n16_d35_k685_p19_s2c_ffr_ci};

pub(super) fn design() -> Result<(BootstrappingPreset, Vec<usize>)> {
    // Degree setting 52 gives a centered degree-56 base.
    let preset = n16_d35_k685_p19_s2c_ffr_ci()?.with_eval_mod_polynomial(16, 52, 2)?;
    let mut scales = preset.scales();
    scales.eval_mod_coeffs_log_delta = 48;
    scales.eval_mod_log_delta = 59;
    Ok((preset.with_scales(scales)?, Vec::new()))
}

/// Phase-two finalist with four fused SlotsToCoeffs factors and the mixed-scale half-angle fit.
pub(super) fn fused_han_ki(stock: BootstrappingPreset) -> Result<(BootstrappingPreset, Vec<usize>)> {
    let preset = stock
        .with_slots_to_coeffs(&[(3, 4), (3, 32), (4, 256), (5, 4096)])?
        .with_first_factor_on_raise(&[(4, 1), (2, 1024), (3, 256), (3, 32), (3, 4)])?
        .with_dsizes(2, 1)?
        .with_eval_mod_polynomial(16, 52, 2)?;
    let mut scales = preset.scales();
    scales.c2s_log_delta = 49;
    scales.eval_mod_coeffs_log_delta = 48;
    scales.eval_mod_log_delta = 58;
    Ok((preset.with_scales(scales)?, vec![0, 1, 2, 3]))
}
