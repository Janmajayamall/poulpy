# Automorphism Refactor Report

## Scope

This document reports the refactor of the `automorphism` family in `poulpy-core`.

It is intended for external review and focuses on:

- the original architectural problem
- the target design
- the concrete code changes
- the migration strategy
- the validation steps
- the implications for downstream backends such as `poulpy-cpu-ref`
- the lessons that should guide the refactor of the remaining `poulpy-core` families

This report reflects the repository state after the automorphism refactor was fully completed and validated.

## Executive Summary

The `automorphism` family in `poulpy-core` has been fully refactored from a two-layer hidden-default model to a single hidden-default model.

Before the refactor, each automorphism subfamily had:

- a public backend-facing dispatch trait in `oep/automorphism.rs`
- a hidden plural trait in `oep/automorphism.rs`
- a hidden singular implementation trait in `default/automorphism/*`

Examples of the old singular traits:

- `GLWEAutomorphismDefault`
- `GLWEAutomorphismKeyAutomorphismDefault`
- `GGSWAutomorphismDefault`

Examples of the plural hidden traits:

- `GLWEAutomorphismDefaults`
- `GGLWEAutomorphismDefaults`
- `GGSWAutomorphismDefaults`

In the old structure, the plural `...Defaults` traits mostly delegated to the singular `...Default` traits. That created:

- extra code surface
- redundant abstraction layers
- harder navigation
- increased coupling between the `oep` and `default` trees
- ambiguity about the intended override boundary for downstream backends

After the refactor:

- the singular `...Default` traits for automorphism are gone
- the default method bodies live directly on the plural hidden `Module<BE>` traits
- the public dispatch remains backend-facing through `AutomorphismImpl<BE>`
- per-method overrides remain possible for concrete backends
- the workspace and `poulpy-cpu-ref` test suite remain green

In addition, a downstream proof was performed by overriding exactly one automorphism method in `poulpy-cpu-ref`, validating the intended customization model.

## Design Goal

The broader architectural goal for `poulpy-core` is:

1. Public APIs should remain backend-facing.
2. Default behavior should unlock automatically from lower-layer capability.
3. Backends should be able to override functionality incrementally.
4. Internal implementation layers should be minimal and navigable.
5. Hidden default traits should define the intended override boundary.

For automorphism specifically, the intended shape is:

- public API traits in `api/`
- delegate forwarding in `delegates/`
- backend dispatch traits in `oep/`
- exactly one hidden `Module<BE>` defaults trait per automorphism subfamily

The intended downstream experience is:

- a backend opts in by implementing an empty hidden defaults trait impl on `Module<BE>`
- a backend overrides one method by implementing just that method
- the backend does not need access to hidden `default::*` internals

## Original Structure

Before the collapse, the automorphism family had three distinct subfamilies:

1. GLWE automorphism
2. GGSW automorphism
3. GGLWE automorphism-key automorphism

Each subfamily was represented twice:

- once in `oep/automorphism.rs` as a plural hidden trait
- once in `default/automorphism/*` as a singular implementation trait

The original hidden singular files were:

- `poulpy-core/src/default/automorphism/glwe_ct.rs`
- `poulpy-core/src/default/automorphism/gglwe_atk.rs`
- `poulpy-core/src/default/automorphism/ggsw_ct.rs`

The original hidden singular traits were:

- `GLWEAutomorphismDefault<BE>`
- `GLWEAutomorphismKeyAutomorphismDefault<BE>`
- `GGSWAutomorphismDefault<BE>`

The plural hidden traits in `poulpy-core/src/oep/automorphism.rs` either extended or delegated to those traits.

This caused a layered hidden structure that looked roughly like:

```text
API trait -> delegate -> backend dispatch trait -> plural hidden defaults trait -> singular hidden default trait -> algorithm body
```

The refactor collapses that to:

```text
API trait -> delegate -> backend dispatch trait -> plural hidden defaults trait -> algorithm body
```

## Final Structure

The final automorphism structure is now:

- `api/automorphism.rs`
  - public traits:
    - `GLWEAutomorphism`
    - `GGSWAutomorphism`
    - `GLWEAutomorphismKeyAutomorphism`

- `delegates/automorphism.rs`
  - `Module<BE>` implementations of those public API traits
  - all forwarding goes through `BE::...`

- `oep/automorphism.rs`
  - public backend dispatch trait:
    - `AutomorphismImpl<BE>`
  - hidden `Module<BE>` defaults traits:
    - `GLWEAutomorphismDefaults<BE>`
    - `GGSWAutomorphismDefaults<BE>`
    - `GGLWEAutomorphismDefaults<BE>`
  - algorithm bodies now live directly in those hidden defaults traits

There is no longer any live `default/automorphism` module.

## Files Removed

The following files were deleted as part of the collapse:

- `poulpy-core/src/default/automorphism/glwe_ct.rs`
- `poulpy-core/src/default/automorphism/gglwe_atk.rs`
- `poulpy-core/src/default/automorphism/ggsw_ct.rs`
- `poulpy-core/src/default/automorphism/mod.rs`

In addition, the old crate-local compatibility shim:

- `pub(crate) mod automorphism { ... }`

was removed from:

- `poulpy-core/src/lib.rs`

because it no longer had any live internal use sites.

## Subfamily-by-Subfamily Changes

### 1. GLWE Automorphism

This was the first localized collapse experiment.

The old structure used:

- `GLWEAutomorphismDefaults<BE>: GLWEAutomorphismDefault<BE>`

The new structure removes that dependency entirely.

`GLWEAutomorphismDefaults<BE>` now:

- carries the required supertraits directly
- contains the default bodies directly
- is the only hidden override surface for GLWE automorphism

The methods moved directly into `GLWEAutomorphismDefaults<BE>` include:

- `glwe_automorphism_tmp_bytes`
- `glwe_automorphism`
- `glwe_automorphism_assign`
- `glwe_automorphism_add`
- `glwe_automorphism_add_assign`
- `glwe_automorphism_sub`
- `glwe_automorphism_sub_negate`
- `glwe_automorphism_sub_assign`
- `glwe_automorphism_sub_negate_assign`

This was the first proof that:

- the singular `default` layer could be removed safely
- the plural hidden defaults trait could own the full implementation surface

### 2. GGLWE Automorphism-Key Automorphism

This subfamily used:

- `GGLWEAutomorphismDefaults<BE>: GLWEAutomorphismKeyAutomorphismDefault<BE>`

The refactor removed that singular dependency and moved the bodies directly into `GGLWEAutomorphismDefaults<BE>`.

The methods now owned directly by `GGLWEAutomorphismDefaults<BE>` are:

- `glwe_automorphism_key_automorphism_tmp_bytes`
- `glwe_automorphism_key_automorphism`
- `glwe_automorphism_key_automorphism_assign`

This subfamily required one additional contract cleanup:

- the API, delegate, and OEP layers were aligned on the same `BE: 's` lifetime contract for the key-automorphism methods

This was not a design flaw in the collapse. It was an inconsistency exposed by moving the real bodies directly into the hidden defaults trait.

### 3. GGSW Automorphism

This was the last singular layer removed.

Unlike GLWE and GGLWE, the old `GGSWAutomorphismDefault` had already been reduced to a very small role. It mainly existed to carry the tmp-bytes computation while the main behavior already composed:

- GLWE automorphism
- GGSW row expansion

The final change was therefore small but important:

- `GGSWAutomorphismDefaults<BE>` was changed to depend directly on:
  - `GLWEAutomorphismDefaults<BE>`
  - `GGSWExpandRowsDefault<BE>`
- the tmp-bytes body was moved directly into `GGSWAutomorphismDefaults<BE>`
- the old singular `GGSWAutomorphismDefault` file was deleted

This also made another hidden adapter removable:

- `OEPModuleRef`

`OEPModuleRef` was no longer used once the GGSW singular layer was gone, so it was deleted too.

## Public Dispatch Model After Refactor

The public dispatch model remains backend-facing.

This is important because the goal was not to move user-facing dispatch onto `Module<BE>`. The goal was:

- keep the public dispatch backend-based
- move the hidden defaults and default bodies onto `Module<BE>`

That final shape is:

```text
Public API trait
  -> Module delegate impl
  -> BE::backend_dispatch_method(...)
  -> blanket AutomorphismImpl<BE> impl
  -> <Module<BE> as *AutomorphismDefaults<BE>>::method(...)
```

This preserves a clean public contract while keeping the override surface localized and function-granular.

## Why the Hidden Defaults Traits Remain

The refactor does not remove:

- `GLWEAutomorphismDefaults`
- `GGSWAutomorphismDefaults`
- `GGLWEAutomorphismDefaults`

Those are intentionally retained.

They are now the correct hidden override surface for downstream backends.

Removing them would destroy the function-by-function override model.

The correct collapse is:

- remove the old singular implementation traits
- keep the hidden plural `Module<BE>` defaults traits

This is what preserves:

- tiny empty opt-in impls
- selective override
- no per-method boilerplate explosion

## Downstream Backend Impact

### Before

Backends such as `poulpy-cpu-ref` and `poulpy-cpu-avx` had to opt into automorphism through hidden defaults traits, but those traits still indirectly depended on singular `default::*` implementation traits.

The override boundary was therefore not fully clean.

### After

Backends now interact only with:

- `GLWEAutomorphismDefaults`
- `GGSWAutomorphismDefaults`
- `GGLWEAutomorphismDefaults`

For `poulpy-cpu-ref`, the normal opt-in shape is:

```rust
impl GLWEAutomorphismDefaults<FFT64Ref> for Module<FFT64Ref> {}
impl GGSWAutomorphismDefaults<FFT64Ref> for Module<FFT64Ref> {}
impl GGLWEAutomorphismDefaults<FFT64Ref> for Module<FFT64Ref> {}
```

That remains extremely small and readable.

## Selective Override Validation

As a final validation step, exactly one method was overridden in `poulpy-cpu-ref`:

- `GLWEAutomorphismDefaults<FFT64Ref>::glwe_automorphism_add`

The override was intentionally written using only the intended public/default surface:

```rust
self.glwe_automorphism(res, a, key, scratch);
self.glwe_add_assign(res, a);
```

This proved several important things:

1. Method-level override still works.
2. The rest of the family remains inherited.
3. The override does not need hidden `default::*` internals.
4. The override does not need private layout field access.
5. The intended customization boundary is real and sufficient.

This is a very strong validation of the design.

## Important Lesson from the Failed First Override Attempt

The first attempt at overriding `glwe_automorphism_add` in `poulpy-cpu-ref` copied the old internal algorithm body.

That failed because the copied code depended on:

- hidden `default::*` traits
- private helper methods
- private layout fields

This failure was useful and informative.

It demonstrates that the new architecture is not merely “compiling after a cleanup.” It genuinely enforces a better layering:

- `poulpy-core` owns the internal implementation kernels
- downstream backends override through the intended public/default surface

This is a feature, not a regression.

The correct rule is:

- if a backend can express a custom behavior by composing the exposed defaults/public operations, it should do that
- if a backend truly needs a lower-level hook, that hook should be made explicit in the intended override surface
- downstream code should not regain access to hidden `default::*` internals by accident

## Validation Matrix

The following validations were performed during and after the refactor:

### Core Compilation

- `cargo check -p poulpy-core --message-format=short`

### Workspace Compilation

- `cargo check --workspace --message-format=short`

### CPU Reference Backend Tests

- `cargo test -p poulpy-cpu-ref --message-format=short`

Final result for `poulpy-cpu-ref`:

- `177 passed; 0 failed`

### Selective Override Validation

After overriding exactly one method in `poulpy-cpu-ref`, the same validations were repeated:

- `cargo test -p poulpy-cpu-ref --message-format=short`
- `cargo check -p poulpy-core --message-format=short`
- `cargo check --workspace --message-format=short`

All passed again.

## What This Refactor Proves

The automorphism refactor proves that the intended architecture is viable in practice, not just in principle.

Specifically, it proves:

1. A `poulpy-core` family can be collapsed from:
   - singular hidden default trait
   - plural hidden defaults trait
   to:
   - one plural hidden defaults trait only

2. Public dispatch can remain backend-facing.

3. Hidden defaults can live on `Module<BE>`.

4. Per-method backend overrides remain possible.

5. Downstream opt-in remains tiny and readable.

6. Workspace stability can be preserved throughout the refactor.

## Why Automorphism Is Now a Good Template

Automorphism is now a strong template for the rest of `poulpy-core` because it includes:

- a non-trivial family with three subfamilies
- interactions with keyswitching and conversion
- tmp-bytes computation
- scratch lifetime handling
- direct behavior methods and helper methods
- downstream override validation

It is therefore not a toy example.

If this pattern works here, it is a credible template for:

- `conversion`
- `keyswitching`
- `encryption`
- `external_product`
- `operations`

with the expected caveat that some families are more coupled than others and may need staged subfamily-by-subfamily collapse.

## Recommended Refactor Rule for Remaining Families

For the rest of `poulpy-core`, the recommended rule is:

1. Identify the plural hidden defaults trait in `oep/*`.
2. Inline the singular hidden implementation bodies into that plural trait.
3. Remove the singular hidden trait file.
4. Keep public dispatch backend-facing.
5. Keep the hidden plural `Module<BE>` defaults trait as the override boundary.
6. Validate by:
   - `cargo check -p poulpy-core`
   - `cargo check --workspace`
   - `cargo test -p poulpy-cpu-ref`
7. For at least one family, validate with a one-method downstream override.

## Risks and Caveats

The automorphism refactor was successful, but it highlighted a few constraints that should be respected in later migrations.

### 1. Signature Alignment Matters

When bodies move into the hidden defaults trait, latent lifetime requirements can become explicit.

This happened for the GGLWE key-automorphism methods.

The fix was to align:

- API signatures
- delegate forwarding signatures
- OEP forwarding signatures

This is a normal refactor requirement, not a conceptual flaw.

### 2. Downstream Overrides Must Not Depend on Hidden Internals

This is now enforced more strongly, which is good.

### 3. Coupled Families Need Staged Collapse

Automorphism was successfully finished, but earlier work on `external_product` showed that some families are more tightly coupled and should likely be collapsed subfamily-by-subfamily rather than in one shot.

## Final Assessment

The automorphism refactor should be considered successful.

It achieved all major goals:

- simpler internal architecture
- smaller code surface
- clearer override boundary
- preserved public dispatch model
- preserved incremental backend customization
- validated behavior under real tests

Most importantly, it does not merely clean up code. It establishes a repeatable architectural pattern for the remainder of the `poulpy-core` refactor.

## Recommended Next Step

Use automorphism as the reference pattern and apply the same collapse to the next `poulpy-core` family, preferably one with moderate complexity and lower coupling than `external_product`, such as:

- `conversion`
- or `decryption`-adjacent remaining default surfaces if any are still layered

The migration should continue family-by-family with the same validation discipline used here.
