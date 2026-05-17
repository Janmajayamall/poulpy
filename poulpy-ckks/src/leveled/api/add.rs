use anyhow::Result;
use poulpy_core::layouts::{GLWEInfos, GLWEToBackendMut, GLWEToBackendRef, LWEInfos};
use poulpy_hal::layouts::{Backend, ScratchArena};

use crate::{CKKSInfos, SetCKKSInfos, oep::CKKSImpl};

pub trait CKKSAddOps<BE: Backend + CKKSImpl<BE>> {
    fn ckks_add_tmp_bytes(&self) -> usize;

    fn ckks_add_into<Dst, A, B>(&self, dst: &mut Dst, a: &A, b: &B, scratch: &mut ScratchArena<'_, BE>) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        B: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;

    fn ckks_add_assign<Dst, A>(&self, dst: &mut Dst, a: &A, scratch: &mut ScratchArena<'_, BE>) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;

    fn ckks_add_pt_vec_tmp_bytes(&self) -> usize;

    fn ckks_add_pt_vec_into<Dst, A, P>(&self, dst: &mut Dst, a: &A, pt: &P, scratch: &mut ScratchArena<'_, BE>) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;

    fn ckks_add_pt_vec_assign<Dst, P>(&self, dst: &mut Dst, pt: &P, scratch: &mut ScratchArena<'_, BE>) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;

    fn ckks_add_pt_const_tmp_bytes(&self) -> usize;

    fn ckks_add_pt_const_into<Dst, A, P>(
        &self,
        dst: &mut Dst,
        a: &A,
        dst_coeff: usize,
        pt: &P,
        pt_coeff: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;

    fn ckks_add_pt_const_assign<Dst, P>(
        &self,
        dst: &mut Dst,
        dst_coeff: usize,
        pt: &P,
        pt_coeff: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait CKKSAddOpsUnsafe<BE: Backend> {
    unsafe fn ckks_add_into_unsafe<Dst, A, B>(
        &self,
        dst: &mut Dst,
        a: &A,
        b: &B,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        B: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;

    unsafe fn ckks_add_assign_unsafe<Dst, A>(&self, dst: &mut Dst, a: &A, scratch: &mut ScratchArena<'_, BE>) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;

    unsafe fn ckks_add_pt_vec_into_unsafe<Dst, A, P>(
        &self,
        dst: &mut Dst,
        a: &A,
        pt: &P,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;

    unsafe fn ckks_add_pt_vec_assign_unsafe<Dst, P>(
        &self,
        dst: &mut Dst,
        pt: &P,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;

    unsafe fn ckks_add_pt_const_into_unsafe<Dst, A, P>(
        &self,
        dst: &mut Dst,
        a: &A,
        dst_coeff: usize,
        pt: &P,
        pt_coeff: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        A: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;

    unsafe fn ckks_add_pt_const_assign_unsafe<Dst, P>(
        &self,
        dst: &mut Dst,
        dst_coeff: usize,
        pt: &P,
        pt_coeff: usize,
        scratch: &mut ScratchArena<'_, BE>,
    ) -> Result<()>
    where
        Dst: GLWEToBackendMut<BE> + GLWEInfos + LWEInfos + CKKSInfos + SetCKKSInfos,
        P: GLWEToBackendRef<BE> + LWEInfos + CKKSInfos;
}
