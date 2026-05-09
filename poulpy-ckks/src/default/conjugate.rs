use anyhow::Result;
use poulpy_core::{
    GLWEShift, ScratchArenaTakeCore,
    layouts::{GGLWEInfos, GGLWEPreparedToBackendRef, GLWEInfos, GLWEToBackendMut, GLWEToBackendRef, GetGaloisElement, LWEInfos},
    oep::GLWEAutomorphismDefaults,
};
use poulpy_hal::layouts::{Backend, ScratchArena};

use crate::{CKKSInfos, SetCKKSInfos, checked_log_budget_sub, ckks_offset_unary};

pub trait CKKSConjugateDefault<BE: Backend> {
    fn ckks_conjugate_tmp_bytes_default<C, K>(&self, ct_infos: &C, key_infos: &K) -> usize
    where
        C: GLWEInfos,
        K: GGLWEInfos,
        Self: GLWEAutomorphismDefaults<BE>,
    {
        <Self as GLWEAutomorphismDefaults<BE>>::glwe_automorphism_tmp_bytes(self, ct_infos, ct_infos, key_infos)
    }

    fn ckks_conjugate_into_default<'s, Dst, Src, K>(
        &self,
        dst: &mut Dst,
        src: &Src,
        key: &K,
        scratch: &mut ScratchArena<'s, BE>,
    ) -> Result<()>
    where
        Self: GLWEAutomorphismDefaults<BE> + GLWEShift<BE>,
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        Src: GLWEToBackendRef<BE> + GLWEInfos + LWEInfos + CKKSInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
        BE: 's,
    {
        let offset = ckks_offset_unary(dst, src);
        if offset != 0 {
            self.glwe_lsh(dst, src, offset, scratch);
            <Self as GLWEAutomorphismDefaults<BE>>::glwe_automorphism_assign(self, dst, key, scratch);
        } else {
            <Self as GLWEAutomorphismDefaults<BE>>::glwe_automorphism(self, dst, src, key, scratch);
        }

        dst.set_meta(src.meta());
        dst.set_log_budget(checked_log_budget_sub("conjugate", dst.log_budget(), offset)?);
        Ok(())
    }

    fn ckks_conjugate_assign_default<'s, Dst, K>(&self, dst: &mut Dst, key: &K, scratch: &mut ScratchArena<'s, BE>) -> Result<()>
    where
        Self: GLWEAutomorphismDefaults<BE>,
        Dst: GLWEToBackendMut<BE> + GLWEInfos,
        K: GetGaloisElement + GGLWEPreparedToBackendRef<BE> + GGLWEInfos,
        ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>,
        BE: 's,
    {
        <Self as GLWEAutomorphismDefaults<BE>>::glwe_automorphism_assign(self, dst, key, scratch);
        Ok(())
    }
}
