# Canonical Dispatch Model for `poulpy-core` Families

## Scope

This document defines the **correct dispatch and override model** for `poulpy-core` operation families.

It is written as a migration guide and design reference, not as a comparison between multiple competing styles.

Its purpose is to answer four questions:

1. What is the canonical layering for a `poulpy-core` family?
2. Where should default algorithm bodies live?
3. How should backends inherit defaults and override individual methods?
4. How should the remaining families be migrated to this model?

The model described here is the one that should be considered canonical going forward.

## Goals

The dispatch architecture for a `poulpy-core` family must satisfy all of the following:

1. Public API remains backend-facing.
2. Default dispatch is unlocked automatically from lower-layer capability.
3. Downstream backends can override **individual methods**.
4. Hidden implementation details do not leak into delegates unnecessarily.
5. There is only one hidden override surface per family.
6. The family remains readable and maintainable after migration.

These goals apply to all families, including:

- automorphism
- conversion
- decryption
- keyswitching
- encryption
- external product
- operations

## Canonical Layering

The canonical layering for a family is:

```text
Public API trait  (api/)
  -> Module<BE> delegate impl  (delegates/)
  -> backend dispatch trait, *Impl<BE>  (oep/)
  -> blanket impl of *Impl<BE> for BE  (oep/)
  -> hidden Module<BE> defaults trait, *Defaults<BE>  (oep/)
  -> algorithm body methods on *Defaults<BE>
```

That is the complete model.

There should **not** be an additional hidden singular `...Default` trait in `default/...` that merely duplicates or forwards the same methods.

## Canonical Trait Roles

Each layer has one responsibility.

### 1. Public API trait

The public API trait defines the user-visible operation surface.

Example:

```rust
pub trait GLWEExternalProduct<BE: Backend> {
    fn glwe_external_product_tmp_bytes<R, A, B>(&self, res_infos: &R, a_infos: &A, b_infos: &B) -> usize
    where
        R: GLWEInfos,
        A: GLWEInfos,
        B: GGSWInfos;

    fn glwe_external_product<'s, R, A, B>(
        &self,
        res: &mut R,
        lhs: &A,
        rhs: &B,
        scratch: &mut ScratchArena<'s, BE>,
    )
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        B: GGSWPreparedToBackendRef<BE> + GGSWInfos,
        ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
        BE: 's;
}
```

This layer should not know anything about hidden defaults traits.

### 2. Delegate impl on `Module<BE>`

The delegate simply forwards public API calls to the backend dispatch trait.

Canonical shape:

```rust
impl<BE> GLWEExternalProduct<BE> for Module<BE>
where
    BE: Backend + GLWEExternalProductImpl<BE>,
{
    fn glwe_external_product_tmp_bytes<R, A, B>(&self, res_infos: &R, a_infos: &A, b_infos: &B) -> usize
    where
        R: GLWEInfos,
        A: GLWEInfos,
        B: GGSWInfos,
    {
        BE::glwe_external_product_tmp_bytes(self, res_infos, a_infos, b_infos)
    }

    fn glwe_external_product<'s, R, A, B>(
        &self,
        res: &mut R,
        lhs: &A,
        rhs: &B,
        scratch: &mut ScratchArena<'s, BE>,
    )
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        B: GGSWPreparedToBackendRef<BE> + GGSWInfos,
        ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
        BE: 's,
    {
        BE::glwe_external_product(self, res, lhs, rhs, scratch)
    }
}
```

Important rule:

- the delegate should depend on `BE: FamilyImpl<BE>`
- it should **not** depend on the same family’s hidden `Module<BE>: FamilyDefaults<BE>` trait

If a delegate needs extra bounds, they should only be for genuinely separate composed families, not for the family’s own hidden defaults surface.

### 3. Backend dispatch trait `*Impl<BE>`

This trait is the backend-facing dispatch contract.

Canonical shape:

```rust
pub unsafe trait GLWEExternalProductImpl<BE: Backend>: Backend {
    fn glwe_external_product_tmp_bytes<R, A, B>(module: &Module<BE>, res_infos: &R, a_infos: &A, b_infos: &B) -> usize
    where
        R: GLWEInfos,
        A: GLWEInfos,
        B: GGSWInfos;

    fn glwe_external_product<'s, R, A, B>(module: &Module<BE>, res: &mut R, a: &A, b: &B, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        B: GGSWPreparedToBackendRef<BE> + GGSWInfos,
        ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
        BE: 's;

    fn glwe_external_product_assign<'s, R, B>(module: &Module<BE>, res: &mut R, b: &B, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        B: GGSWPreparedToBackendRef<BE> + GGSWInfos,
        ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
        BE: 's;
}
```

Important rule:

- `*Impl<BE>` should be **abstract**
- it should **not** carry default method bodies
- it should **not** repeat the family implementation logic directly

Its job is only to define the backend dispatch contract.

### 4. Blanket impl of `*Impl<BE> for BE`

The blanket impl owns the connection between the backend dispatch trait and the hidden defaults trait.

Canonical shape:

```rust
#[allow(private_bounds)]
unsafe impl<BE: Backend> GLWEExternalProductImpl<BE> for BE
where
    Module<BE>: GLWEExternalProductDefaults<BE>,
{
    fn glwe_external_product_tmp_bytes<R, A, B>(module: &Module<BE>, res_infos: &R, a_infos: &A, b_infos: &B) -> usize
    where
        R: GLWEInfos,
        A: GLWEInfos,
        B: GGSWInfos,
    {
        module.glwe_external_product_tmp_bytes(res_infos, a_infos, b_infos)
    }

    fn glwe_external_product<'s, R, A, B>(module: &Module<BE>, res: &mut R, a: &A, b: &B, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        B: GGSWPreparedToBackendRef<BE> + GGSWInfos,
        ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
        BE: 's,
    {
        module.glwe_external_product(res, a, b, scratch)
    }

    fn glwe_external_product_assign<'s, R, B>(module: &Module<BE>, res: &mut R, b: &B, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        B: GGSWPreparedToBackendRef<BE> + GGSWInfos,
        ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
        BE: 's,
    {
        module.glwe_external_product_assign(res, b, scratch)
    }
}
```

Important rule:

- the hidden `Defaults<BE>` requirement belongs here
- not on the public API trait
- not on the public API method signatures
- ideally not on the same family’s delegate impl

### 5. Hidden `Module<BE>` defaults trait

This trait is the intended override surface.

Canonical shape:

```rust
#[doc(hidden)]
#[allow(private_bounds)]
pub trait GLWEExternalProductDefaults<BE: Backend>:
    Sized
    + GLWEExternalProductInternal<BE>
    + VecZnxDftBytesOf
    + VecZnxIdftApply<BE>
    + VecZnxIdftApplyTmpBytes
    + VecZnxBigBytesOf
    + VecZnxBigNormalize<BE>
    + VecZnxBigNormalizeTmpBytes
    + GLWENormalizeDefault<BE>
where
    for<'s> ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
{
    fn glwe_external_product_tmp_bytes<R, A, B>(&self, res_infos: &R, a_infos: &A, b_infos: &B) -> usize
    where
        R: GLWEInfos,
        A: GLWEInfos,
        B: GGSWInfos,
    {
        /* full algorithm body */
    }

    fn glwe_external_product<'s, R, A, B>(&self, res: &mut R, a: &A, b: &B, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        B: GGSWPreparedToBackendRef<BE> + GGSWInfos,
        ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
        BE: 's,
    {
        /* full algorithm body */
    }
}
```

Important rule:

- this is the **only hidden override surface**
- this trait should own the real default bodies
- there should not be a second singular `...Default` layer below it

## What Must Not Exist in the Final Model

The final canonical model should **not** contain a duplicated hidden implementation layer like:

- `GLWEAutomorphismDefault`
- `GGSWAutomorphismDefault`
- `GLWEAutomorphismKeyAutomorphismDefault`
- `GGLWEExternalProductDefault`
- `GGSWExternalProductDefault`

These singular traits were historically useful as migration scaffolding, but they are not part of the desired end state.

If such a trait still exists, the family is not fully collapsed yet.

## Canonical Backend Experience

### Empty opt-in

If the backend wants the default implementation unchanged:

```rust
impl GLWEExternalProductDefaults<FFT64Ref> for Module<FFT64Ref> {}
```

That is the normal inheritance path.

### Single-method override

If the backend wants to override exactly one method:

```rust
impl GLWEExternalProductDefaults<FFT64Ref> for Module<FFT64Ref> {
    fn glwe_external_product<'s, R, A, B>(
        &self,
        res: &mut R,
        a: &A,
        b: &B,
        scratch: &mut ScratchArena<'s, FFT64Ref>,
    )
    where
        R: GLWEToBackendMut<FFT64Ref> + GLWEInfos,
        A: GLWEToBackendRef<FFT64Ref> + GLWEInfos,
        B: GGSWPreparedToBackendRef<FFT64Ref> + GGSWInfos,
        ScratchArena<'s, FFT64Ref>: ScratchArenaTakeCore<'s, FFT64Ref>,
        FFT64Ref: 's,
    {
        /* custom implementation */
    }
}
```

Every other method remains inherited.

This is the required selective-override behavior.

## Override Rule

The intended backend override path is:

- override methods on `Module<BE>: FamilyDefaults<BE>`

The intended backend override path is **not**:

- implementing the backend `FamilyImpl<BE>` trait directly
- depending on hidden `default::*` singular traits
- reusing private layout internals from `poulpy-core`

This distinction is important.

Once a family is properly migrated:

- `FamilyImpl<BE>` is just dispatch
- `FamilyDefaults<BE>` is the only supported hidden customization surface

## A Concrete Selective Override Example

The automorphism refactor validated the model by overriding exactly one method in `poulpy-cpu-ref`.

The override was:

```rust
impl GLWEAutomorphismDefaults<FFT64Ref> for Module<FFT64Ref> {
    fn glwe_automorphism_add<'s, R, A, K>(
        &self,
        res: &mut R,
        a: &A,
        key: &K,
        scratch: &mut ScratchArena<'s, FFT64Ref>,
    )
    where
        R: GLWEToBackendMut<FFT64Ref> + GLWEInfos,
        A: GLWEToBackendRef<FFT64Ref> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<FFT64Ref> + GGLWEInfos,
        FFT64Ref: 's,
    {
        self.glwe_automorphism(res, a, key, scratch);
        self.glwe_add_assign(res, a);
    }
}
```

That override is valuable as a model because:

- it overrides only one method
- it leaves the rest of the family inherited
- it uses the exposed public/default surface
- it does **not** depend on old hidden internal kernels

This is the intended downstream experience.

## Migration Instructions

The following process should be applied family-by-family.

### Step 1. Identify the current dispatch layers

For the family under migration, locate:

- the public API trait in `api/`
- the delegate impl in `delegates/`
- the backend dispatch trait in `oep/`
- the hidden plural defaults trait in `oep/`
- any singular `...Default` trait(s) in `default/`

### Step 2. Decide the family boundaries

If the family is naturally split into subfamilies, migrate each one separately.

Examples:

- automorphism:
  - GLWE
  - GGSW
  - GGLWE key automorphism

This is recommended for more coupled families.

### Step 3. Move default bodies into the plural `...Defaults` trait

If a singular trait currently owns the real bodies:

```rust
trait GLWEAutomorphismDefault<BE> { ... }
trait GLWEAutomorphismDefaults<BE>: GLWEAutomorphismDefault<BE> { ... }
```

then migrate to:

```rust
trait GLWEAutomorphismDefaults<BE> { /* full bodies here */ }
```

The plural hidden defaults trait should own:

- tmp-bytes methods
- core algorithm methods
- assign variants
- additive/subtractive variants

### Step 4. Remove the singular hidden trait

Once no callers depend on it, delete:

- the singular trait definition
- its impl for `Module<BE>`
- its module re-export
- any `crate::...` compatibility shim that only existed to surface it

### Step 5. Make the backend dispatch trait abstract

If the family currently puts default bodies directly on `*Impl<BE>`, move those bodies out.

The final shape should be:

- abstract methods on `*Impl<BE>`
- forwarding lives in the blanket impl
- algorithm bodies live only in `*Defaults<BE>`

### Step 6. Move the `Defaults<BE>` requirement into the blanket impl

The family’s own hidden defaults bound should not appear on the public delegate unless there is a strong technical reason that cannot be removed.

Preferred:

```rust
impl<BE> FamilyApi<BE> for Module<BE>
where
    BE: Backend + FamilyImpl<BE>,
```

Not preferred:

```rust
impl<BE> FamilyApi<BE> for Module<BE>
where
    BE: Backend + FamilyImpl<BE>,
    Module<BE>: FamilyDefaults<BE>,
```

### Step 7. Keep only genuine cross-family requirements

If a family composes another family, it is acceptable for delegates or OEP methods to mention the other family’s requirements.

For example:

- automorphism’s GGSW slice may legitimately depend on conversion behavior

But the family should not leak its **own** hidden defaults trait unless unavoidable.

### Step 8. Validate selective override

For at least one migrated family, add a concrete backend override of exactly one method and verify:

- compilation succeeds
- the rest of the family remains inherited
- the override can be expressed through the intended surface

This is the strongest proof that the architecture is correct.

## Final State Requirements

A family should be considered fully migrated only if all of the following are true.

### Structural requirements

1. Public API trait exists in `api/`.
2. `Module<BE>` delegate exists in `delegates/`.
3. Backend dispatch trait `FamilyImpl<BE>` exists in `oep/`.
4. Blanket impl `unsafe impl<BE: Backend> FamilyImpl<BE> for BE` exists.
5. Hidden plural `FamilyDefaults<BE>` trait exists and owns the full default bodies.
6. No singular `FamilyDefault<BE>` trait remains for that family.

### Layering requirements

1. Public API methods do not mention the family’s hidden defaults trait.
2. Delegates do not mention the family’s hidden defaults trait unless strictly unavoidable.
3. Blanket impl owns the connection between dispatch and hidden defaults.
4. Backends opt in through `impl FamilyDefaults<BE> for Module<BE>`.

### Override requirements

1. Empty backend impl inherits the whole family.
2. Overriding one method does not require reimplementing the whole family.
3. Overriding one method does not require access to deleted singular internal traits.
4. Overriding one method does not require private layout fields from `poulpy-core`.

### Validation requirements

1. `cargo check -p poulpy-core --message-format=short`
2. `cargo check --workspace --message-format=short`
3. `cargo test -p poulpy-cpu-ref --message-format=short`

must all pass after the migration.

## Example of a Fully Finished Family

A fully finished family should look like this conceptually:

```text
api/family.rs
  - public trait FamilyApi<BE>

delegates/family.rs
  - impl FamilyApi<BE> for Module<BE>
  - bounds only on BE: FamilyImpl<BE> and genuine cross-family requirements

oep/family.rs
  - unsafe trait FamilyImpl<BE>: Backend { abstract methods only }
  - #[doc(hidden)] trait FamilyDefaults<BE> { full default bodies }
  - blanket impl FamilyImpl<BE> for BE where Module<BE>: FamilyDefaults<BE>

default/family/*
  - no singular FamilyDefault layer remains
  - ideally no family-specific module remains unless it still owns unrelated internal helpers
```

## Recommendation

This is the dispatch model that should be used for the remainder of the `poulpy-core` migration.

The practical rule is:

- one public backend dispatch trait
- one hidden `Module<BE>` defaults trait
- one blanket impl connecting them
- no extra singular hidden default layer

This is the model that preserves:

- clean public dispatch
- automatic default unlock
- function-by-function override
- minimal code surface
- maintainability for future refactors

## Immediate Next Use

The next family migrated after automorphism should follow this document directly.

For each remaining family:

1. identify the current singular hidden traits
2. move their bodies into the plural hidden defaults trait
3. make `*Impl<BE>` abstract if needed
4. move the family’s own hidden-default requirement behind the blanket impl
5. validate with one downstream one-method override where useful

That is the correct migration path.
