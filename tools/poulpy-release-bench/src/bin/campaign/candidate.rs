//! Starting implementation for source evolution. Other editable Rust modules supply
//! the transforms, fusion kernel and polynomial fit used by this implementation.
use anyhow::Result;
use poulpy_ckks::presets::bootstrapping::{BootstrappingPreset, n16_d35_k685_p19_s2c_ffr_ci};

pub(super) fn design() -> Result<(BootstrappingPreset, Vec<usize>)> {
    let preset = n16_d35_k685_p19_s2c_ffr_ci()?;
    let fused = Vec::new();
    Ok((preset, fused))
}
