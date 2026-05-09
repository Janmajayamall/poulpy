use crate::default::add::CKKSAddDefault;

use anyhow::Result;
use poulpy_core::{GLWEAdd, GLWENormalize, GLWEShift, ScratchArenaTakeCore, layouts::LWEInfos};
use poulpy_hal::{
    api::{VecZnxRshAddCoeffIntoBackend, VecZnxRshAddIntoBackend, VecZnxRshTmpBytes},
    layouts::{Backend, Module, ScratchArena},
};

use crate::{CKKSInfos, GLWEToBackendMut, GLWEToBackendRef, SetCKKSInfos};

/// # Safety
///
/// Implementations must satisfy the contracts of all trait methods, including
/// any HAL-level invariants (alignment, layout, scratch sizing) implied by the
/// associated method signatures.
pub unsafe trait CKKSAddImpl<BE: Backend>: Backend {
    fn ckks_add_tmp_bytes(module: &Module<BE>) -> usize;
    fn ckks_add_into<Dst, A, B>(
        module: &Module<BE>,
        dst: &mut Dst,
        a: &A,
        b: &B,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        B: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;
    fn ckks_add_assign<Dst, A>(module: &Module<BE>, dst: &mut Dst, a: &A, scratch: &mut ScratchArena<'_, BE>) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + CKKSInfos;
    fn ckks_add_pt_vec_znx_tmp_bytes(module: &Module<BE>) -> usize;
    fn ckks_add_pt_vec_znx_into<Dst, A, P>(
        module: &Module<BE>,
        dst: &mut Dst,
        a: &A,
        pt_znx: &P,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;
    fn ckks_add_pt_vec_znx_assign<Dst, P>(
        module: &Module<BE>,
        dst: &mut Dst,
        pt_znx: &P,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;
    fn ckks_add_pt_const_tmp_bytes(module: &Module<BE>) -> usize;
    fn ckks_add_pt_const_znx_into<Dst, A, P>(
        module: &Module<BE>,
        dst: &mut Dst,
        a: &A,
        dst_coeff: usize,
        pt_znx: &P,
        pt_coeff: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;
    fn ckks_add_pt_const_znx_assign<Dst, P>(
        module: &Module<BE>,
        dst: &mut Dst,
        dst_coeff: usize,
        pt_znx: &P,
        pt_coeff: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;
}

#[allow(private_bounds)]
unsafe impl<BE: Backend> CKKSAddImpl<BE> for BE
where
    BE: poulpy_hal::oep::HalVecZnxImpl<BE>,
    Module<BE>: crate::default::add::CKKSAddDefault<BE>
        + GLWEAdd<BE>
        + GLWENormalize<BE>
        + GLWEShift<BE>
        + VecZnxRshAddCoeffIntoBackend<BE>
        + VecZnxRshAddIntoBackend<BE>
        + VecZnxRshTmpBytes,
    for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
{
    fn ckks_add_tmp_bytes(module: &Module<BE>) -> usize {
        module.ckks_add_tmp_bytes_default()
    }

    fn ckks_add_into<Dst, A, B>(
        module: &Module<BE>,
        dst: &mut Dst,
        a: &A,
        b: &B,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        B: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
    {
        module.ckks_add_into_default(dst, a, b, scratch)
    }

    fn ckks_add_assign<Dst, A>(module: &Module<BE>, dst: &mut Dst, a: &A, scratch: &mut ScratchArena<'_, BE>) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + CKKSInfos,
    {
        module.ckks_add_assign_default(dst, a, scratch)
    }

    fn ckks_add_pt_vec_znx_tmp_bytes(module: &Module<BE>) -> usize {
        module.ckks_add_pt_vec_znx_tmp_bytes_default()
    }

    fn ckks_add_pt_vec_znx_into<Dst, A, P>(
        module: &Module<BE>,
        dst: &mut Dst,
        a: &A,
        pt_znx: &P,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
    {
        crate::default::add::CKKSAddDefault::ckks_add_pt_vec_znx_into_default(module, dst, a, pt_znx, scratch)
    }

    fn ckks_add_pt_vec_znx_assign<Dst, P>(
        module: &Module<BE>,
        dst: &mut Dst,
        pt_znx: &P,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
    {
        module.ckks_add_pt_vec_znx_assign_default(dst, pt_znx, scratch)
    }

    fn ckks_add_pt_const_tmp_bytes(module: &Module<BE>) -> usize {
        module.ckks_add_pt_const_tmp_bytes_default()
    }

    fn ckks_add_pt_const_znx_into<Dst, A, P>(
        module: &Module<BE>,
        dst: &mut Dst,
        a: &A,
        dst_coeff: usize,
        pt_znx: &P,
        pt_coeff: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
    {
        module.ckks_add_pt_const_znx_into_default(dst, a, dst_coeff, pt_znx, pt_coeff, scratch)
    }

    fn ckks_add_pt_const_znx_assign<Dst, P>(
        module: &Module<BE>,
        dst: &mut Dst,
        dst_coeff: usize,
        pt_znx: &P,
        pt_coeff: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
    {
        module.ckks_add_pt_const_znx_assign_default(dst, dst_coeff, pt_znx, pt_coeff, scratch)
    }
}
