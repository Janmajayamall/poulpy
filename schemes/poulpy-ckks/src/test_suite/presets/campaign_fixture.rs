//! Deterministic fixtures and key accounting for the bootstrap campaign.
use super::*;
use crate::layouts::BootstrappingKeySet;
use poulpy_hal::layouts::DataView;

pub struct CampaignFixture {
    pub key_seed: Option<u64>,
    pub encryption_seed: Option<u64>,
    pub vector_seed: Option<u64>,
    pub radius: f64,
}

pub(super) fn fixture_source(seed: Option<u64>, stream: u8) -> Source {
    let mut bytes = [stream; 32];
    if let Some(value) = seed {
        bytes[0..8].copy_from_slice(&value.to_le_bytes());
        bytes[8] = 0xfe;
    }
    Source::new(bytes)
}

impl CampaignFixture {
    pub(super) fn vector(&self, slots: usize) -> (Vec<f64>, Vec<f64>) {
        if self.vector_seed.is_none() {
            let (mut re, mut im) = test_vector_1::<f64>(slots);
            for x in re.iter_mut().chain(&mut im) {
                *x *= self.radius;
            }
            return (re, im);
        }
        let mut rng = fixture_source(self.vector_seed, 9);
        (0..slots)
            .map(|i| {
                let angle = rng.next_f64(0.0, std::f64::consts::TAU);
                let radius = self.radius
                    * match i % 4 {
                        0 => 1.0,
                        1 => 1.0 - 2f64.powi(-20),
                        _ => rng.next_f64(0.0, 1.0).sqrt(),
                    };
                (radius * angle.cos(), radius * angle.sin())
            })
            .unzip()
    }
}

// Walk the owned bundle, whose entries are distinct allocations. Lookup views can alias those entries.
macro_rules! visit_keys {
    ($keys:ident, $visit:ident) => {{
        for k in $keys.rotation_keys.values() {
            $visit(k);
        }
        if let Some(k) = &$keys.tensor_key {
            $visit(k);
        }
        if let Some(k) = &$keys.ci_tensor_key {
            $visit(k);
        }
        if let Some((a, b)) = &$keys.encapsulation_keys {
            $visit(a);
            if let Some(k) = b {
                $visit(k);
            }
        }
        for k in $keys.first_factor_keys.values() {
            $visit(k);
        }
        if let Some((a, b)) = &$keys.conversion_keys {
            $visit(a);
            $visit(b);
        }
    }};
}

pub(super) fn serialized_bytes<D: HostDataRef, W: ZnxWord>(keys: &BootstrappingKeySet<D, W>) -> usize {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0 += buf.len();
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Counter(0);
    let mut visit = |k: &dyn KeySize| k.count(&mut count);
    trait KeySize {
        fn count(&self, dst: &mut Counter);
    }
    impl<T: WriterTo> KeySize for T {
        fn count(&self, dst: &mut Counter) {
            self.write_to(dst).unwrap();
        }
    }
    visit_keys!(keys, visit);
    count.0
}

pub(super) fn key_bytes<BE: Backend>(keys: &BootstrappingKeysPrepared<BE::OwnedBuf, BE>) -> (usize, usize) {
    fn size<BE: Backend, R: Ring>(k: &impl GGLWEPreparedToBackendRef<BE, R>) -> usize {
        BE::len_bytes_ref(k.to_backend_ref().data().data())
    }
    let rotations = keys.rotation_keys.values().map(size::<BE, Standard>).sum::<usize>();
    let mut total = rotations;
    if let Some(k) = &keys.tensor_key {
        total += size(k);
    }
    if let Some(k) = &keys.ci_tensor_key {
        total += size(k);
    }
    if let Some((a, b)) = &keys.encapsulation_keys {
        total += size(a);
        if let Some(k) = b {
            total += size(k);
        }
    }
    total += keys.first_factor_keys.values().map(size::<BE, Standard>).sum::<usize>();
    if let Some((a, b)) = &keys.conversion_keys {
        total += size(a) + size(b);
    }
    (total, rotations)
}
