use poulpy_hal::layouts::{Backend, Module, ScratchArena};

use crate::{
    ScratchArenaTakeCore,
    layouts::{
        GGLWEInfos, GGLWEToBackendMut, GGLWEToBackendRef, GGSWInfos, GGSWToBackendMut, GGSWToBackendRef, GLWEInfos,
        GLWEToBackendMut, GLWEToBackendRef, GetGaloisElement, SetGaloisElement,
        prepared::{GGLWEPreparedToBackendRef, GGLWEToGGSWKeyPreparedToBackendRef},
    },
};

/// Backend hook for automorphism-family operations.
///
/// # Safety
/// Implementors must preserve the semantics, scratch requirements, aliasing
/// guarantees, and backend bit-parity contract expected by end-to-end pipelines.
#[allow(private_bounds)]
pub unsafe trait AutomorphismImpl<BE: Backend>: Backend {
    fn glwe_automorphism_tmp_bytes<R, A, K>(module: &Module<BE>, res_infos: &R, a_infos: &A, key_infos: &K) -> usize
    where
        R: GLWEInfos,
        A: GLWEInfos,
        K: GGLWEInfos;

    fn glwe_automorphism<'s, R, A, K>(module: &Module<BE>, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_assign<'s, R, K>(module: &Module<BE>, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_add<'s, R, A, K>(module: &Module<BE>, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_add_assign<'s, R, K>(module: &Module<BE>, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_sub<'s, R, A, K>(module: &Module<BE>, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_sub_negate<'s, R, A, K>(
        module: &Module<BE>,
        res: &mut R,
        a: &A,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_sub_assign<'s, R, K>(module: &Module<BE>, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_sub_negate_assign<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn ggsw_automorphism_tmp_bytes<R, A, K, T>(
        module: &Module<BE>,
        res_infos: &R,
        a_infos: &A,
        key_infos: &K,
        tsk_infos: &T,
    ) -> usize
    where
        R: GGSWInfos,
        A: GGSWInfos,
        K: GGLWEInfos,
        T: GGLWEInfos;

    fn ggsw_automorphism<'s, R, A, K, T>(
        module: &Module<BE>,
        res: &mut R,
        a: &A,
        key: &K,
        tsk: &T,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGSWToBackendMut<BE> + GGSWInfos,
        A: GGSWToBackendRef<BE> + GGSWInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        T: GGLWEToGGSWKeyPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn ggsw_automorphism_assign<'s, R, K, T>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        tsk: &T,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGSWToBackendMut<BE> + GGSWInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        T: GGLWEToGGSWKeyPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_key_automorphism_tmp_bytes<R, A, K>(
        module: &Module<BE>,
        res_infos: &R,
        a_infos: &A,
        key_infos: &K,
    ) -> usize
    where
        R: GGLWEInfos,
        A: GGLWEInfos,
        K: GGLWEInfos;

    fn glwe_automorphism_key_automorphism<'s, R, A, K>(
        module: &Module<BE>,
        res: &mut R,
        a: &A,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + SetGaloisElement + GGLWEInfos,
        A: GGLWEToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_key_automorphism_assign<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + SetGaloisElement + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        BE: 's;
}

/// Override surface for the GLWE-automorphism sub-family.
///
/// This trait is intentionally **abstract**: it carries no HAL supertraits and no default
/// method bodies. A backend may provide its own kernels for some or all methods without
/// satisfying any HAL trait.
///
/// To inherit the reference algorithms, forward each method to the corresponding
/// `glwe_automorphism_defaults::*` free function — those carry the HAL bounds in their
/// own `where` clauses, so the requirement only kicks in for methods that actually use
/// the default implementation.
#[doc(hidden)]
#[allow(private_bounds)]
pub trait GLWEAutomorphismDefaults<BE: Backend>
where
    for<'s> ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
{
    fn glwe_automorphism_tmp_bytes<R, A, K>(&self, res_infos: &R, a_infos: &A, key_infos: &K) -> usize
    where
        R: GLWEInfos,
        A: GLWEInfos,
        K: GGLWEInfos;

    fn glwe_automorphism<'s, R, A, K>(&self, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_assign<'s, R, K>(&self, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_add<'s, R, A, K>(&self, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_add_assign<'s, R, K>(&self, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_sub<'s, R, A, K>(&self, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_sub_negate<'s, R, A, K>(&self, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_sub_assign<'s, R, K>(&self, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_sub_negate_assign<'s, R, K>(&self, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;
}

/// Override surface for the GGSW-automorphism sub-family.
///
/// Abstract: no HAL supertraits, no default method bodies. See
/// [`crate::default::automorphism::ggsw`] for reference algorithms a backend may forward to.
#[doc(hidden)]
#[allow(private_bounds)]
pub trait GGSWAutomorphismDefaults<BE: Backend>
where
    for<'s> ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
{
    fn ggsw_automorphism_tmp_bytes<R, A, K, T>(&self, res_infos: &R, a_infos: &A, key_infos: &K, tsk_infos: &T) -> usize
    where
        R: GGSWInfos,
        A: GGSWInfos,
        K: GGLWEInfos,
        T: GGLWEInfos;

    fn ggsw_automorphism<'s, R, A, K, T>(&self, res: &mut R, a: &A, key: &K, tsk: &T, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GGSWToBackendMut<BE> + GGSWInfos,
        A: GGSWToBackendRef<BE> + GGSWInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        T: GGLWEToGGSWKeyPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn ggsw_automorphism_assign<'s, R, K, T>(&self, res: &mut R, key: &K, tsk: &T, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GGSWToBackendMut<BE> + GGSWInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        T: GGLWEToGGSWKeyPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;
}

/// Override surface for the GGLWE key-automorphism sub-family.
///
/// Abstract: no HAL supertraits, no default method bodies. See
/// [`crate::default::automorphism::gglwe`] for reference algorithms a backend may forward to.
#[doc(hidden)]
#[allow(private_bounds)]
pub trait GGLWEAutomorphismDefaults<BE: Backend>
where
    for<'s> ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
{
    fn glwe_automorphism_key_automorphism_tmp_bytes<R, A, K>(&self, res_infos: &R, a_infos: &A, key_infos: &K) -> usize
    where
        R: GGLWEInfos,
        A: GGLWEInfos,
        K: GGLWEInfos;

    fn glwe_automorphism_key_automorphism<'s, R, A, K>(&self, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GGLWEToBackendMut<BE> + SetGaloisElement + GGLWEInfos,
        A: GGLWEToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_key_automorphism_assign<'s, R, K>(&self, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GGLWEToBackendMut<BE> + SetGaloisElement + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        BE: 's;
}

#[allow(private_bounds)]
unsafe impl<BE> AutomorphismImpl<BE> for BE
where
    BE: Backend,
    Module<BE>: GLWEAutomorphismDefaults<BE> + GGSWAutomorphismDefaults<BE> + GGLWEAutomorphismDefaults<BE>,
    for<'s> ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
{
    fn glwe_automorphism_tmp_bytes<R, A, K>(module: &Module<BE>, res_infos: &R, a_infos: &A, key_infos: &K) -> usize
    where
        R: GLWEInfos,
        A: GLWEInfos,
        K: GGLWEInfos,
    {
        module.glwe_automorphism_tmp_bytes(res_infos, a_infos, key_infos)
    }

    fn glwe_automorphism<'s, R, A, K>(module: &Module<BE>, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism(res, a, key, scratch)
    }

    fn glwe_automorphism_assign<'s, R, K>(module: &Module<BE>, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_assign(res, key, scratch)
    }

    fn glwe_automorphism_add<'s, R, A, K>(module: &Module<BE>, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_add(res, a, key, scratch)
    }

    fn glwe_automorphism_add_assign<'s, R, K>(module: &Module<BE>, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_add_assign(res, key, scratch)
    }

    fn glwe_automorphism_sub<'s, R, A, K>(module: &Module<BE>, res: &mut R, a: &A, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_sub(res, a, key, scratch)
    }

    fn glwe_automorphism_sub_negate<'s, R, A, K>(
        module: &Module<BE>,
        res: &mut R,
        a: &A,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        A: GLWEToBackendRef<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_sub_negate(res, a, key, scratch)
    }

    fn glwe_automorphism_sub_assign<'s, R, K>(module: &Module<BE>, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_sub_assign(res, key, scratch)
    }

    fn glwe_automorphism_sub_negate_assign<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_sub_negate_assign(res, key, scratch)
    }

    fn ggsw_automorphism_tmp_bytes<R, A, K, T>(
        module: &Module<BE>,
        res_infos: &R,
        a_infos: &A,
        key_infos: &K,
        tsk_infos: &T,
    ) -> usize
    where
        R: GGSWInfos,
        A: GGSWInfos,
        K: GGLWEInfos,
        T: GGLWEInfos,
    {
        module.ggsw_automorphism_tmp_bytes(res_infos, a_infos, key_infos, tsk_infos)
    }

    fn ggsw_automorphism<'s, R, A, K, T>(
        module: &Module<BE>,
        res: &mut R,
        a: &A,
        key: &K,
        tsk: &T,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGSWToBackendMut<BE> + GGSWInfos,
        A: GGSWToBackendRef<BE> + GGSWInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        T: GGLWEToGGSWKeyPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.ggsw_automorphism(res, a, key, tsk, scratch)
    }

    fn ggsw_automorphism_assign<'s, R, K, T>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        tsk: &T,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGSWToBackendMut<BE> + GGSWInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        T: GGLWEToGGSWKeyPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.ggsw_automorphism_assign(res, key, tsk, scratch)
    }

    fn glwe_automorphism_key_automorphism_tmp_bytes<R, A, K>(
        module: &Module<BE>,
        res_infos: &R,
        a_infos: &A,
        key_infos: &K,
    ) -> usize
    where
        R: GGLWEInfos,
        A: GGLWEInfos,
        K: GGLWEInfos,
    {
        module.glwe_automorphism_key_automorphism_tmp_bytes(res_infos, a_infos, key_infos)
    }

    fn glwe_automorphism_key_automorphism<'s, R, A, K>(
        module: &Module<BE>,
        res: &mut R,
        a: &A,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + SetGaloisElement + GGLWEInfos,
        A: GGLWEToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_key_automorphism(res, a, key, scratch)
    }

    fn glwe_automorphism_key_automorphism_assign<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + SetGaloisElement + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_key_automorphism_assign(res, key, scratch)
    }
}

/// Implements [`GLWEAutomorphismDefaults`] for `Module<$be>` by forwarding every method to
/// the corresponding [`glwe_automorphism_defaults`] free function.
///
/// Equivalent to writing all 9 forwarders by hand. For partial override (custom kernel for one
/// or a few methods + defaults for the rest), write the impl block manually instead.
#[macro_export]
macro_rules! impl_glwe_automorphism_defaults_full {
    ($be:ty) => {
        impl $crate::oep::GLWEAutomorphismDefaults<$be> for ::poulpy_hal::layouts::Module<$be> {
            fn glwe_automorphism_tmp_bytes<R, A, K>(&self, res_infos: &R, a_infos: &A, key_infos: &K) -> usize
            where
                R: $crate::layouts::GLWEInfos,
                A: $crate::layouts::GLWEInfos,
                K: $crate::layouts::GGLWEInfos,
            {
                $crate::default::automorphism::glwe::glwe_automorphism_tmp_bytes_default::<$be, _, _, _, _>(
                    self, res_infos, a_infos, key_infos,
                )
            }

            fn glwe_automorphism<'s, R, A, K>(
                &self,
                res: &mut R,
                a: &A,
                key: &K,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GLWEToBackendMut<$be> + $crate::layouts::GLWEInfos,
                A: $crate::layouts::GLWEToBackendRef<$be> + $crate::layouts::GLWEInfos,
                K: $crate::layouts::GetGaloisElement
                    + $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::glwe::glwe_automorphism_default::<$be, _, _, _, _>(self, res, a, key, scratch)
            }

            fn glwe_automorphism_assign<'s, R, K>(
                &self,
                res: &mut R,
                key: &K,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GLWEToBackendMut<$be> + $crate::layouts::GLWEInfos,
                K: $crate::layouts::GetGaloisElement
                    + $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::glwe::glwe_automorphism_assign_default::<$be, _, _, _>(self, res, key, scratch)
            }

            fn glwe_automorphism_add<'s, R, A, K>(
                &self,
                res: &mut R,
                a: &A,
                key: &K,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GLWEToBackendMut<$be> + $crate::layouts::GLWEInfos,
                A: $crate::layouts::GLWEToBackendRef<$be> + $crate::layouts::GLWEInfos,
                K: $crate::layouts::GetGaloisElement
                    + $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::glwe::glwe_automorphism_add_default::<$be, _, _, _, _>(self, res, a, key, scratch)
            }

            fn glwe_automorphism_add_assign<'s, R, K>(
                &self,
                res: &mut R,
                key: &K,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GLWEToBackendMut<$be> + $crate::layouts::GLWEInfos,
                K: $crate::layouts::GetGaloisElement
                    + $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::glwe::glwe_automorphism_add_assign_default::<$be, _, _, _>(self, res, key, scratch)
            }

            fn glwe_automorphism_sub<'s, R, A, K>(
                &self,
                res: &mut R,
                a: &A,
                key: &K,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GLWEToBackendMut<$be> + $crate::layouts::GLWEInfos,
                A: $crate::layouts::GLWEToBackendRef<$be> + $crate::layouts::GLWEInfos,
                K: $crate::layouts::GetGaloisElement
                    + $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::glwe::glwe_automorphism_sub_default::<$be, _, _, _, _>(self, res, a, key, scratch)
            }

            fn glwe_automorphism_sub_negate<'s, R, A, K>(
                &self,
                res: &mut R,
                a: &A,
                key: &K,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GLWEToBackendMut<$be> + $crate::layouts::GLWEInfos,
                A: $crate::layouts::GLWEToBackendRef<$be> + $crate::layouts::GLWEInfos,
                K: $crate::layouts::GetGaloisElement
                    + $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::glwe::glwe_automorphism_sub_negate_default::<$be, _, _, _, _>(
                    self, res, a, key, scratch,
                )
            }

            fn glwe_automorphism_sub_assign<'s, R, K>(
                &self,
                res: &mut R,
                key: &K,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GLWEToBackendMut<$be> + $crate::layouts::GLWEInfos,
                K: $crate::layouts::GetGaloisElement
                    + $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::glwe::glwe_automorphism_sub_assign_default::<$be, _, _, _>(self, res, key, scratch)
            }

            fn glwe_automorphism_sub_negate_assign<'s, R, K>(
                &self,
                res: &mut R,
                key: &K,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GLWEToBackendMut<$be> + $crate::layouts::GLWEInfos,
                K: $crate::layouts::GetGaloisElement
                    + $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::glwe::glwe_automorphism_sub_negate_assign_default::<$be, _, _, _>(
                    self, res, key, scratch,
                )
            }
        }
    };
}

/// Implements [`GGSWAutomorphismDefaults`] for `Module<$be>` by forwarding every method to
/// the corresponding [`ggsw_automorphism_defaults`] free function.
#[macro_export]
macro_rules! impl_ggsw_automorphism_defaults_full {
    ($be:ty) => {
        impl $crate::oep::GGSWAutomorphismDefaults<$be> for ::poulpy_hal::layouts::Module<$be> {
            fn ggsw_automorphism_tmp_bytes<R, A, K, T>(&self, res_infos: &R, a_infos: &A, key_infos: &K, tsk_infos: &T) -> usize
            where
                R: $crate::layouts::GGSWInfos,
                A: $crate::layouts::GGSWInfos,
                K: $crate::layouts::GGLWEInfos,
                T: $crate::layouts::GGLWEInfos,
            {
                $crate::default::automorphism::ggsw::ggsw_automorphism_tmp_bytes_default::<$be, _, _, _, _, _>(
                    self, res_infos, a_infos, key_infos, tsk_infos,
                )
            }

            fn ggsw_automorphism<'s, R, A, K, T>(
                &self,
                res: &mut R,
                a: &A,
                key: &K,
                tsk: &T,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GGSWToBackendMut<$be> + $crate::layouts::GGSWInfos,
                A: $crate::layouts::GGSWToBackendRef<$be> + $crate::layouts::GGSWInfos,
                K: $crate::layouts::GetGaloisElement
                    + $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GGLWEInfos,
                T: $crate::layouts::prepared::GGLWEToGGSWKeyPreparedToBackendRef<$be> + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::ggsw::ggsw_automorphism_default::<$be, _, _, _, _, _>(
                    self, res, a, key, tsk, scratch,
                )
            }

            fn ggsw_automorphism_assign<'s, R, K, T>(
                &self,
                res: &mut R,
                key: &K,
                tsk: &T,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GGSWToBackendMut<$be> + $crate::layouts::GGSWInfos,
                K: $crate::layouts::GetGaloisElement
                    + $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GGLWEInfos,
                T: $crate::layouts::prepared::GGLWEToGGSWKeyPreparedToBackendRef<$be> + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::ggsw::ggsw_automorphism_assign_default::<$be, _, _, _, _>(
                    self, res, key, tsk, scratch,
                )
            }
        }
    };
}

/// Implements [`GGLWEAutomorphismDefaults`] for `Module<$be>` by forwarding every method to
/// the corresponding [`gglwe_automorphism_defaults`] free function.
#[macro_export]
macro_rules! impl_gglwe_automorphism_defaults_full {
    ($be:ty) => {
        impl $crate::oep::GGLWEAutomorphismDefaults<$be> for ::poulpy_hal::layouts::Module<$be> {
            fn glwe_automorphism_key_automorphism_tmp_bytes<R, A, K>(&self, res_infos: &R, a_infos: &A, key_infos: &K) -> usize
            where
                R: $crate::layouts::GGLWEInfos,
                A: $crate::layouts::GGLWEInfos,
                K: $crate::layouts::GGLWEInfos,
            {
                $crate::default::automorphism::gglwe::glwe_automorphism_key_automorphism_tmp_bytes_default::<$be, _, _, _, _>(
                    self, res_infos, a_infos, key_infos,
                )
            }

            fn glwe_automorphism_key_automorphism<'s, R, A, K>(
                &self,
                res: &mut R,
                a: &A,
                key: &K,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GGLWEToBackendMut<$be> + $crate::layouts::SetGaloisElement + $crate::layouts::GGLWEInfos,
                A: $crate::layouts::GGLWEToBackendRef<$be> + $crate::layouts::GetGaloisElement + $crate::layouts::GGLWEInfos,
                K: $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GetGaloisElement
                    + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::gglwe::glwe_automorphism_key_automorphism_default::<$be, _, _, _, _>(
                    self, res, a, key, scratch,
                )
            }

            fn glwe_automorphism_key_automorphism_assign<'s, R, K>(
                &self,
                res: &mut R,
                key: &K,
                scratch: &mut ::poulpy_hal::layouts::ScratchArena<'s, $be>,
            ) where
                R: $crate::layouts::GGLWEToBackendMut<$be>
                    + $crate::layouts::SetGaloisElement
                    + $crate::layouts::GetGaloisElement
                    + $crate::layouts::GGLWEInfos,
                K: $crate::layouts::prepared::GGLWEPreparedToBackendRef<$be>
                    + $crate::layouts::GetGaloisElement
                    + $crate::layouts::GGLWEInfos,
                $be: 's,
            {
                $crate::default::automorphism::gglwe::glwe_automorphism_key_automorphism_assign_default::<$be, _, _, _>(
                    self, res, key, scratch,
                )
            }
        }
    };
}
