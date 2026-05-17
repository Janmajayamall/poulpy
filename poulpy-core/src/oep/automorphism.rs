use poulpy_hal::layouts::{Backend, Module, ScratchArena};

use crate::{
    ScratchArenaTakeCore,
    automorphism::{GGSWAutomorphismDefault, GLWEAutomorphismDefault, GLWEAutomorphismKeyAutomorphismDefault},
    layouts::{
        GGLWEInfos, GGLWEToBackendMut, GGLWEToBackendRef, GGSWInfos, GGSWToBackendMut, GGSWToBackendRef, GLWEInfos,
        GLWEToBackendMut, GLWEToBackendRef, GetGaloisElement, SetGaloisElement,
        prepared::{GGLWEPreparedToBackendRef, GGLWEToGGSWKeyPreparedToBackendRef},
    },
};

/// Backend hook for automorphism-family operations.
///
/// # Safety
/// Implementors must preserve the semantics, scratch requirements, and aliasing
/// guarantees expected by the default forwarding layer for every exposed method.
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
        T: GGLWEInfos,
        BE: AutomorphismImpl<BE> + crate::oep::ConversionImpl<BE>;

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
        BE: AutomorphismImpl<BE> + crate::oep::ConversionImpl<BE>;

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
        BE: AutomorphismImpl<BE> + crate::oep::ConversionImpl<BE>;

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
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos;

    fn glwe_automorphism_key_automorphism_assign<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + SetGaloisElement + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos;
}

#[doc(hidden)]
#[allow(private_bounds)]
pub trait AutomorphismDefaults<BE: Backend>: Backend {
    fn glwe_automorphism_tmp_bytes_default<R, A, K>(module: &Module<BE>, res_infos: &R, a_infos: &A, key_infos: &K) -> usize
    where
        R: GLWEInfos,
        A: GLWEInfos,
        K: GGLWEInfos;

    fn glwe_automorphism_default<'s, R, A, K>(
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

    fn glwe_automorphism_assign_default<'s, R, K>(module: &Module<BE>, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_add_default<'s, R, A, K>(
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

    fn glwe_automorphism_add_assign_default<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_sub_default<'s, R, A, K>(
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

    fn glwe_automorphism_sub_negate_default<'s, R, A, K>(
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

    fn glwe_automorphism_sub_assign_default<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn glwe_automorphism_sub_negate_assign_default<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's;

    fn ggsw_automorphism_tmp_bytes_default<R, A, K, T>(
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
        BE: AutomorphismImpl<BE> + crate::oep::ConversionImpl<BE>,
        Module<BE>: GGSWAutomorphismDefault<BE>;

    fn ggsw_automorphism_default<'s, R, A, K, T>(
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
        BE: AutomorphismImpl<BE> + crate::oep::ConversionImpl<BE>,
        BE: 's,
        Module<BE>: GGSWAutomorphismDefault<BE>;

    fn ggsw_automorphism_assign_default<'s, R, K, T>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        tsk: &T,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGSWToBackendMut<BE> + GGSWInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        T: GGLWEToGGSWKeyPreparedToBackendRef<BE> + GGLWEInfos,
        BE: AutomorphismImpl<BE> + crate::oep::ConversionImpl<BE>,
        BE: 's,
        Module<BE>: GGSWAutomorphismDefault<BE>;

    fn glwe_automorphism_key_automorphism_tmp_bytes_default<R, A, K>(
        module: &Module<BE>,
        res_infos: &R,
        a_infos: &A,
        key_infos: &K,
    ) -> usize
    where
        R: GGLWEInfos,
        A: GGLWEInfos,
        K: GGLWEInfos,
        Module<BE>: GLWEAutomorphismKeyAutomorphismDefault<BE>;

    fn glwe_automorphism_key_automorphism_default<'s, R, A, K>(
        module: &Module<BE>,
        res: &mut R,
        a: &A,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + SetGaloisElement + GGLWEInfos,
        A: GGLWEToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        Module<BE>: GLWEAutomorphismKeyAutomorphismDefault<BE>;

    fn glwe_automorphism_key_automorphism_assign_default<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + SetGaloisElement + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        Module<BE>: GLWEAutomorphismKeyAutomorphismDefault<BE>;
}

#[allow(private_bounds)]
impl<BE: Backend> AutomorphismDefaults<BE> for BE
where
    Module<BE>: GLWEAutomorphismDefault<BE>,
    for<'s> ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
{
    fn glwe_automorphism_tmp_bytes_default<R, A, K>(module: &Module<BE>, res_infos: &R, a_infos: &A, key_infos: &K) -> usize
    where
        R: GLWEInfos,
        A: GLWEInfos,
        K: GGLWEInfos,
    {
        module.glwe_automorphism_tmp_bytes_default(res_infos, a_infos, key_infos)
    }

    fn glwe_automorphism_default<'s, R, A, K>(
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
        module.glwe_automorphism_default(res, a, key, scratch)
    }

    fn glwe_automorphism_assign_default<'s, R, K>(module: &Module<BE>, res: &mut R, key: &K, scratch: &mut ScratchArena<'s, BE>)
    where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_assign_default(res, key, scratch)
    }

    fn glwe_automorphism_add_default<'s, R, A, K>(
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
        module.glwe_automorphism_add_default(res, a, key, scratch)
    }

    fn glwe_automorphism_add_assign_default<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_add_assign_default(res, key, scratch)
    }

    fn glwe_automorphism_sub_default<'s, R, A, K>(
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
        module.glwe_automorphism_sub_default(res, a, key, scratch)
    }

    fn glwe_automorphism_sub_negate_default<'s, R, A, K>(
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
        module.glwe_automorphism_sub_negate_default(res, a, key, scratch)
    }

    fn glwe_automorphism_sub_assign_default<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_sub_assign_default(res, key, scratch)
    }

    fn glwe_automorphism_sub_negate_assign_default<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        BE: 's,
    {
        module.glwe_automorphism_sub_negate_assign_default(res, key, scratch)
    }

    fn ggsw_automorphism_tmp_bytes_default<R, A, K, T>(
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
        BE: AutomorphismImpl<BE> + crate::oep::ConversionImpl<BE>,
        Module<BE>: GGSWAutomorphismDefault<BE>,
    {
        module.ggsw_automorphism_tmp_bytes_default(res_infos, a_infos, key_infos, tsk_infos)
    }

    fn ggsw_automorphism_default<'s, R, A, K, T>(
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
        BE: AutomorphismImpl<BE> + crate::oep::ConversionImpl<BE>,
        BE: 's,
        Module<BE>: GGSWAutomorphismDefault<BE>,
    {
        module.ggsw_automorphism_default(res, a, key, tsk, scratch)
    }

    fn ggsw_automorphism_assign_default<'s, R, K, T>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        tsk: &T,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGSWToBackendMut<BE> + GGSWInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        T: GGLWEToGGSWKeyPreparedToBackendRef<BE> + GGLWEInfos,
        BE: AutomorphismImpl<BE> + crate::oep::ConversionImpl<BE>,
        BE: 's,
        Module<BE>: GGSWAutomorphismDefault<BE>,
    {
        module.ggsw_automorphism_assign_default(res, key, tsk, scratch)
    }

    fn glwe_automorphism_key_automorphism_tmp_bytes_default<R, A, K>(
        module: &Module<BE>,
        res_infos: &R,
        a_infos: &A,
        key_infos: &K,
    ) -> usize
    where
        R: GGLWEInfos,
        A: GGLWEInfos,
        K: GGLWEInfos,
        Module<BE>: GLWEAutomorphismKeyAutomorphismDefault<BE>,
    {
        module.glwe_automorphism_key_automorphism_tmp_bytes_default(res_infos, a_infos, key_infos)
    }

    fn glwe_automorphism_key_automorphism_default<'s, R, A, K>(
        module: &Module<BE>,
        res: &mut R,
        a: &A,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: crate::layouts::GGLWEToBackendMut<BE> + SetGaloisElement + GGLWEInfos,
        A: crate::layouts::GGLWEToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        Module<BE>: GLWEAutomorphismKeyAutomorphismDefault<BE>,
    {
        module.glwe_automorphism_key_automorphism_default(res, a, key, scratch)
    }

    fn glwe_automorphism_key_automorphism_assign_default<'s, R, K>(
        module: &Module<BE>,
        res: &mut R,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: crate::layouts::GGLWEToBackendMut<BE> + SetGaloisElement + GetGaloisElement + GGLWEInfos,
        K: GGLWEPreparedToBackendRef<BE> + GetGaloisElement + GGLWEInfos,
        Module<BE>: GLWEAutomorphismKeyAutomorphismDefault<BE>,
    {
        module.glwe_automorphism_key_automorphism_assign_default(res, key, scratch)
    }
}
