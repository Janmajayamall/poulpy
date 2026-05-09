# Backend-Agnostic Cleanup Handoff

Date: 2026-05-04

## Goal

Assess the workspace against the backend-agnostic cleanup direction:

- Avoid unnecessary type casting when a call can be generic.
- Remove legacy typecasting patterns that are no longer needed.
- Keep API, OEP, delegates, and defaults consistent.

## Current Assessment

The workspace is in a much better state than before the generic migration, but it is not fully clean yet.

The intended layering is sound:

- Public API, OEP, and delegates should be generic over layout-like inputs.
- Concrete backend views are acceptable only near actual HAL calls, private low-level helpers, layout adapters, and scratch views.
- Higher-level code should not need to manually call `.to_backend_ref()` or `.to_backend_mut()` just to reach another core-level method.
- There should be no `&&` double-reference workaround and no trait impls added only to support that pattern.

Latest known verification before this assessment:

- `cargo check --workspace --message-format=short` passed.
- `cargo test -p poulpy-cpu-ref -p poulpy-cpu-avx --message-format=short` passed.
- A search for obvious `&&` patterns over core/hal/cpu-ref/bin-fhe/ckks/bench found no remaining matches.

## Crate Status

### `poulpy-ckks`

Mostly clean.

The CKKS API/OEP/delegate layers are largely generic and routed consistently:

- API methods take generic layout inputs.
- OEP tends to route through `BE::ckks_*`.
- Delegates tend to route through `CKKS*Oep::*`.

Remaining `.to_backend_ref()` / `.to_backend_mut()` calls in CKKS defaults are mostly acceptable because they are close to implementation details or HAL/core calls. They should still be reviewed case-by-case, but CKKS is not the main source of architectural friction right now.

### `poulpy-core`

Mixed.

Many API/OEP/delegate surfaces are now generic, especially around automorphism, keyswitching, conversion, and external product. However, `operations` still has a concrete-view leak.

The clearest issue is `poulpy-core/src/api/operations.rs`, where `GLWENormalize` still exposes concrete backend views through methods such as:

- `glwe_maybe_cross_normalize_to_ref`
- `glwe_maybe_cross_normalize_to_mut`

That concrete surface then propagates into:

- `poulpy-core/src/oep/operations.rs`
- `poulpy-core/src/delegates/operations.rs`

This is inconsistent with the rest of the migration. These methods should either become generic, or be moved out of public API/OEP/delegate surfaces into private/default-only helpers.

There are also places in `poulpy-core/src/oep/operations.rs` where values are converted to backend views before calling another core/default-level method. If the callee is not a HAL method, that conversion should usually be pushed lower.

### `poulpy-bin-fhe`

Least clean.

Several higher-level algorithm paths still materialize backend views and manually slice rows before calling generic operations. Examples to inspect first:

- `poulpy-bin-fhe/src/bdd_arithmetic/blind_rotation.rs`
- `poulpy-bin-fhe/src/bdd_arithmetic/eval.rs`
- `poulpy-bin-fhe/src/blind_rotation/algorithms/cggi/algorithm.rs`
- `poulpy-bin-fhe/src/circuit_bootstrapping/circuit.rs`

Patterns such as this are architectural hotspots:

```rust
let res = &mut res.to_backend_mut();
let a = &a.to_backend_ref();
```

When the following call is a core-level generic operation, this is unnecessary leakage. If the code truly needs low-level row access, that access should be isolated behind a generic row/subview helper or clearly kept inside a low-level implementation boundary.

### `poulpy-hal`

HAL is the right place for concrete backend views, but there are still suspicious compatibility impls.

The `HostBytesBackend` conversion impls, for example:

```rust
impl VecZnxToBackendRef<crate::layouts::HostBytesBackend> for VecZnx<&[u8]>
```

look like compatibility patches for CPU-ref helper code rather than a clean backend-generic design. They should be treated as temporary until the CPU-ref helper paths stop requiring host-specific default conversions.

## Main Remaining Problems

### 1. Concrete Normalize API

`GLWENormalize` still exposes concrete backend views in `poulpy-core/src/api/operations.rs`.

This is the highest-priority cleanup because it is a public/core API inconsistency and forces concrete view handling upward.

Recommended direction:

- Make normalize helpers generic if they are real public operations.
- Otherwise move them into private/default-only implementation helpers.
- Keep concrete views only at the point where HAL layout data is actually accessed.

### 2. Early Type Conversion In OEP

Some OEP code still performs `.to_backend_ref()` / `.to_backend_mut()` before dispatching to methods that could be generic.

Rule of thumb:

- OEP should preserve generic types when calling core/default operations.
- Defaults may convert when they reach HAL operations or direct layout access.
- HAL-facing helpers may use concrete views freely.

### 3. Legacy View-Heavy Algorithms In `poulpy-bin-fhe`

Several bin-fhe algorithms still look like they predate the generic API cleanup.

The most visible issue is manually converting whole objects to backend views just to access rows or sub-objects, then passing those subviews into generic calls.

Recommended direction:

- Add or use generic row/subview access where appropriate.
- Keep direct backend slicing inside low-level implementation helpers.
- Avoid exposing `GGSWPreparedBackendMut`, `GLWEBackendMut`, etc. from high-level traits unless the trait is intentionally HAL-facing.

### 4. Host-Specific Layout Conversion Impl

The `HostBytesBackend` impls in `poulpy-hal` are likely temporary compatibility glue.

Recommended direction:

- Identify which CPU-ref helpers still require them.
- Make those helpers use the backend-generic path.
- Remove the host-specific impls once no longer needed.

## Consistency Rules Going Forward

Use these rules when reviewing or adding code:

1. API methods should take generic layout inputs.
2. OEP should dispatch generically and avoid materializing concrete backend views unless calling HAL directly.
3. Delegates may route through `BE::method` or `Trait::method`, but should not introduce concrete type requirements.
4. Defaults may use concrete backend views when they access layout internals or call HAL.
5. HAL APIs should use concrete backend views.
6. No `&&` workaround should be introduced.
7. No trait impl should exist only to make a double-reference workaround compile.
8. If `.to_backend_ref()` or `.to_backend_mut()` appears outside defaults/HAL/layout code, it should be reviewed.

## Suggested Cleanup Order

1. Fix `GLWENormalize` in `poulpy-core/src/api/operations.rs`.
2. Clean `poulpy-core/src/oep/operations.rs` so conversions happen only at HAL/default boundaries.
3. Sweep `poulpy-bin-fhe`, starting with `bdd_arithmetic/blind_rotation.rs`.
4. Review and remove host-specific compatibility impls in `poulpy-hal` once CPU-ref no longer depends on them.
5. Re-run:

```bash
cargo check --workspace --message-format=short
cargo test -p poulpy-cpu-ref -p poulpy-cpu-avx --message-format=short
```

## Bottom Line

The migration direction is correct and the workspace is substantially cleaner than before. The remaining work is not broad conceptual repair; it is targeted cleanup of a few concrete-view leaks and legacy algorithm paths.

The next best technical target is `GLWENormalize`, because fixing that will remove the most visible inconsistency across API/OEP/delegates.
