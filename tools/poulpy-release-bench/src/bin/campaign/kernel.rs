//! Full S2C-first execution, with first-call diagnostics outside the timing loop.
use super::{BE, fusion::Factor};
use poulpy_ckks::{
    CKKSInfos, SetCKKSInfos,
    api::{CKKSBootstrappingOps, CKKSCopyOps, CKKSDecryptOps, CKKSEncodingHostOps, CKKSPow2Ops, CKKSSubOps},
    layouts::{CKKSCiphertextAlloc, CKKSCiphertextOwned, CKKSDFTFactors, CKKSPlaintextAlloc},
    presets::bootstrapping::BootstrappingPreset,
    reference::dft::eval::eval_factor,
    test_suite::presets::{Bootstrap, BootstrappingPresetRun, CampaignFixture},
};
use poulpy_core::layouts::BackendGLWESecretPrepared;
use poulpy_core::{
    GLWEBytesOf,
    api::GLWESecretPreparedFactory,
    layouts::{LWEInfos, SetK},
};
use poulpy_hal::layouts::{Module, ScratchArena};
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc, time::Instant};

fn difference(
    m: &Module<BE>,
    a: &CKKSCiphertextOwned<BE>,
    b: &mut CKKSCiphertextOwned<BE>,
    sk: &BackendGLWESecretPrepared<BE>,
    scratch: &mut ScratchArena<'_, BE>,
) -> poulpy_ckks::CKKSResult<f64> {
    m.ckks_sub_assign(b, a, scratch)?;
    let slots = a.n().as_usize() / 2;
    let mut pt = m.ckks_plaintext_alloc_from_slots(slots, a.base2k(), (a.log_delta() + 8).into());
    pt.set_meta(a.meta());
    m.ckks_decrypt(&mut pt, b, sk, scratch)?;
    let (mut re, mut im) = (vec![0.0f64; slots], vec![0.0f64; slots]);
    m.ckks_decode_slots_host_into(&pt, &mut re, &mut im, scratch)?;
    Ok(re
        .into_iter()
        .chain(im)
        .map(|x| if x.is_finite() { x.abs() } else { f64::INFINITY })
        .fold(0.0, f64::max))
}

pub fn setup(
    preset: BootstrappingPreset,
    fixture: CampaignFixture,
    indices: Vec<usize>,
    profile: Rc<RefCell<Vec<Value>>>,
) -> BootstrappingPresetRun<BE> {
    let seed = fixture.key_seed;
    BootstrappingPresetRun::setup_with_kernel(preset, false, Some(fixture), move |m, sk, preset, ctx, keys| {
        let s2c = preset.plan().slots_to_coeffs();
        let mut factors: Vec<_> = (0..s2c.schedule().steps.len()).map(|_| None).collect();
        let (mut wire, mut prepared) = (0, 0);
        for &i in &indices {
            let f = Factor::new(m, sk, preset, i, seed, indices.len()).expect("compile fusion key");
            wire += f.serialized;
            prepared += f.prepared;
            factors[i] = Some(f);
        }
        let input_layout = preset.input_layout();
        let mut c = m.ckks_ciphertext_alloc_from_glwe_infos(&input_layout);
        let boot_layout = preset.bootstrap_layout();
        let mut z = m.ckks_ciphertext_alloc_from_glwe_infos(&boot_layout);
        let workspace = m.glwe_bytes_of_from_infos(&input_layout) + m.glwe_bytes_of_from_infos(&boot_layout);
        let mut verify_sk = (!indices.is_empty()).then(|| {
            let mut p = m.glwe_secret_prepared_alloc_from_infos(sk);
            m.glwe_secret_prepare(&mut p, sk);
            p
        });
        let mut first = true;
        let kernel: Bootstrap<BE> = Box::new(move |m, out, input, scratch| {
            c.set_k(input.k());
            c.set_meta(input.meta());
            z.set_k(boot_layout.k());
            z.set_meta(input.meta());
            let run = ctx.run(input.meta());
            m.ckks_double_into(&mut c, input, scratch)?;
            let dft = ctx.slots_to_coeffs();
            for (i, (factor, plan)) in dft.factors().zip(dft.factor_plans()).enumerate() {
                let before = c.k().as_usize();
                let reference = if first && factors[i].is_some() {
                    let mut tmp = m.ckks_ciphertext_alloc_from_glwe_infos(&input_layout);
                    m.ckks_copy(&mut tmp, &c, scratch)?;
                    let start = Instant::now();
                    eval_factor(m, &mut tmp, factor, plan, &keys, scratch)?;
                    Some((tmp, start.elapsed().as_nanos() as u64))
                } else {
                    None
                };
                let start = first.then(Instant::now);
                if let Some(f) = &factors[i] {
                    f.apply(m, &mut c, scratch);
                } else {
                    eval_factor(m, &mut c, factor, plan, &keys, scratch)?;
                }
                if let Some(start) = start {
                    let mut row = json!({"stage":"s2c","index":i,"k_in":before,"k_out":c.k().as_usize(),"ns":start.elapsed().as_nanos() as u64,
                        "diagonals":factor.giant_steps.iter().map(|g|g.diagonals.len()).sum::<usize>()});
                    if let Some((mut expected, ns)) = reference {
                        row["unfused_ns"] = json!(ns);
                        let error = difference(m, &c, &mut expected, verify_sk.as_ref().unwrap(), scratch)?;
                        row["max_fused_error"] = json!(error);
                        row["added_prepared_bytes"] = json!(factors[i].as_ref().unwrap().prepared);
                    }
                    profile.borrow_mut().push(row);
                }
            }
            let start = first.then(Instant::now);
            let before = c.k().as_usize();
            m.ckks_mod_raise(&mut z, &mut c, &run, &keys, scratch)?;
            if let Some(s) = start {
                profile.borrow_mut().push(json!({"stage":"raise_and_c2s","index":0,"k_in":before,"k_out":z.k().as_usize(),"ns":s.elapsed().as_nanos() as u64}));
            }
            let dft = ctx.coeffs_to_slots();
            for (i, (factor, plan)) in dft.factors().zip(dft.factor_plans()).enumerate() {
                let before = z.k().as_usize();
                let start = first.then(Instant::now);
                eval_factor(m, &mut z, factor, plan, &keys, scratch)?;
                if let Some(s) = start {
                    profile.borrow_mut().push(json!({"stage":"c2s","index":i+1,"k_in":before,"k_out":z.k().as_usize(),"ns":s.elapsed().as_nanos() as u64}));
                }
            }
            m.ckks_bootstrap_eval_mod(out, &z, &run, &keys, scratch)?;
            verify_sk.take();
            first = false;
            Ok(0)
        });
        (kernel, wire, prepared, workspace)
    })
}
