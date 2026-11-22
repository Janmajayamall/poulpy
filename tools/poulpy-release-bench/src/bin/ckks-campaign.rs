//! Trusted measurement entry point. Fixed controls and compiled Rust candidates share the same checks.
#[path = "campaign/candidate.rs"]
mod candidate;
#[path = "campaign/fusion.rs"]
mod fusion;
#[path = "campaign/kernel.rs"]
mod kernel;
use anyhow::{Result, ensure};
use poulpy_ckks::{
    layouts::EvalModType,
    polynomial::SplitStrategy,
    presets::bootstrapping::{BootstrappingPreset, n16_d35_k685_p19_s2c_ffr_ci},
    test_suite::{
        helpers::PrecisionStats,
        presets::{CampaignFixture, preset_for_backend},
    },
};
use poulpy_cpu_avx::NTT4x30AvxRayon as BE;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{cell::RefCell, io::stdin, rc::Rc, time::Instant};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Control {
    Stock,
    Pow2,
    FusedFirst,
    HanKi,
    Residue,
}

impl Control {
    fn design(self) -> Result<(BootstrappingPreset, Vec<usize>)> {
        let stock = n16_d35_k685_p19_s2c_ffr_ci()?;
        match self {
            Self::Stock => Ok((stock, vec![])),
            Self::Pow2 => Ok((stock.with_power_of_two_keys()?, vec![])),
            Self::FusedFirst => Ok((stock, vec![0])),
            Self::HanKi | Self::Residue => {
                let (family, degree, steps) = match self {
                    Self::HanKi => (EvalModType::CosHKEven, 30, 2),
                    _ => (EvalModType::CenteredResidue, 31, 0),
                };
                let preset = stock.with_campaign_recipe(
                    &[(3, 1), (3, 2048), (3, 256), (3, 32), (3, 4)],
                    &[(3, 4), (4, 32), (4, 512), (4, 8192)],
                    stock.scales(),
                    family,
                    degree,
                    steps,
                    SplitStrategy::MinDepth,
                    false,
                )?;
                Ok((preset, vec![]))
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Plan,
    Measure,
    Finalist,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    mode: Mode,
    control: Option<Control>,
    radius: f64,
    key_seed: Option<u64>,
    encryption_seed: Option<u64>,
    vector_seed: Option<u64>,
    warmup: usize,
    iters: usize,
}

const FIRST_BOOTSTRAP_BITS: f64 = 19.0;
const SECOND_BOOTSTRAP_BITS: f64 = 18.5;

fn precision((real, imag): (PrecisionStats, PrecisionStats), required_bits: f64) -> Value {
    let max_error = (-required_bits).exp2();
    json!({
        "required_bits": required_bits,
        "max_abs_err": [real.worst_err, imag.worst_err],
        "min_bits": [real.min_log2_prec, imag.min_log2_prec],
        "worst_slots": [real.worst_idx, imag.worst_idx],
        "passes": real.worst_err <= max_error && imag.worst_err <= max_error,
    })
}

fn main() -> Result<()> {
    let req: Request = serde_json::from_reader(stdin().lock())?;
    ensure!(req.radius.is_finite() && req.radius > 0.0, "invalid domain radius");
    let (preset, indices) = match req.control {
        Some(control) => control.design()?,
        None => candidate::design()?,
    };
    let preset = preset_for_backend::<BE>(&preset)?;
    let stock = n16_d35_k685_p19_s2c_ffr_ci()?;
    ensure!(
        preset.dense_secret_hamming_weight() == stock.dense_secret_hamming_weight()
            && preset.sparse_secret_hamming_weight() == stock.sparse_secret_hamming_weight()
            && preset.max_dense_modulus() == stock.max_dense_modulus()
            && preset.max_sparse_modulus() == stock.max_sparse_modulus()
            && preset.log_msg_ratio() == stock.log_msg_ratio()
            && preset.plan().eval_mod().f_mod_interval == stock.plan().eval_mod().f_mod_interval
            && preset.plan().pipeline() == stock.plan().pipeline()
            && preset.plan().log_gap() == stock.plan().log_gap()
            && preset.plan().eval_mod_ring() == stock.plan().eval_mod_ring()
            && preset.plan().coeffs_to_slots().first_factor_on_raise()
            && preset.plan().coeffs_to_slots().log_slots() == 15
            && preset.plan().slots_to_coeffs().log_slots() == 15,
        "fixed bootstrap contract changed"
    );
    ensure!(
        indices
            .iter()
            .all(|&i| i < preset.plan().slots_to_coeffs().schedule().steps.len())
            && indices.windows(2).all(|p| p[0] < p[1]),
        "invalid fusion placement"
    );
    ensure!(
        (preset.n(), preset.input_k(), preset.output_k(), preset.log_delta()) == (65536, 160, 685, 35),
        "public widths/scale/packing changed"
    );
    ensure!(
        preset.base2k() <= preset.max_backend_base2k::<BE>(128),
        "unsafe arithmetic radix"
    );
    let em = preset.plan().eval_mod();
    let (worst, zero, sensitivity) = em.approximation_errors(req.radius)?;
    let mut out = json!({"qualified":false, "raised_bits":preset.bootstrap_k(), "base2k":preset.base2k(),
        "compiled_depth":em.eval_depth(),"wrap_max_error":worst,"zero_wrap_error":zero,"max_sampled_slope":sensitivity,"wrap_interval":em.f_mod_interval});
    if !matches!(req.mode, Mode::Plan) {
        let fixture = CampaignFixture {
            key_seed: req.key_seed,
            encryption_seed: req.encryption_seed,
            vector_seed: req.vector_seed,
            radius: req.radius,
        };
        let start = Instant::now();
        let profile = Rc::new(RefCell::new(Vec::new()));
        let mut run = kernel::setup(preset, fixture, indices, profile.clone());
        out["factors"] = json!(*profile.borrow());
        run.synchronize();
        out["setup_ns"] = json!(start.elapsed().as_nanos() as u64);
        let (serialized, prepared, rotations, scratch) = run.campaign_bytes;
        out["serialized_key_bytes"] = json!(serialized);
        out["prepared_key_bytes"] = json!(prepared);
        out["rotation_key_bytes"] = json!(rotations);
        out["scratch_bytes"] = json!(scratch);
        let layout = run.measured_layout();
        ensure!(layout == (160, 685, 35, 35, 32768), "measured layout mismatch: {layout:?}");
        out["layout"] = json!(layout);
        out["spendable_bits"] = json!(layout.1 - layout.0);
        out["precision"] = precision(run.precision(), FIRST_BOOTSTRAP_BITS);
        let mut ok = out["precision"]["passes"] == true
            && profile.borrow().iter().all(|r| {
                r.get("max_fused_error")
                    .is_none_or(|v| v.as_f64().is_some_and(|x| x <= 2f64.powi(-19)))
            });
        if ok && matches!(req.mode, Mode::Finalist) {
            run.spend_and_refresh()?;
            out["second_precision"] = precision(run.precision(), SECOND_BOOTSTRAP_BITS);
            ok = out["second_precision"]["passes"] == true;
        }
        if ok && matches!(req.mode, Mode::Measure) {
            let mut sample = || {
                let start = Instant::now();
                run.bootstrap();
                start.elapsed().as_nanos() as u64
            };
            out["warmup_ns"] = json!((0..req.warmup).map(|_| sample()).collect::<Vec<_>>());
            out["samples_ns"] = json!((0..req.iters).map(|_| sample()).collect::<Vec<_>>());
        }
        out["qualified"] = json!(ok);
    }
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use poulpy_ckks::test_suite::helpers::precision_stats;

    #[test]
    fn first_and_second_bootstrap_enforce_their_own_inclusive_bounds() {
        let stats = |error: f64| precision_stats(&[error], &[0.0], 35);
        for (bits, limit) in [
            (FIRST_BOOTSTRAP_BITS, 2f64.powi(-19)),
            (SECOND_BOOTSTRAP_BITS, 2f64.powi(-19) * std::f64::consts::SQRT_2),
        ] {
            assert_eq!(precision((stats(limit), stats(limit)), bits)["passes"], true);
            assert_eq!(precision((stats(limit.next_up()), stats(0.0)), bits)["passes"], false);
            assert_eq!(precision((stats(0.0), stats(limit.next_up())), bits)["passes"], false);
        }
    }
}
