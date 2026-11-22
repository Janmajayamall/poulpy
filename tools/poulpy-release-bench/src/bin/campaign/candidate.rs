//! Starting implementation for source evolution. Other editable Rust modules supply
//! the transforms, fusion kernel and polynomial fit used by this implementation.
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
