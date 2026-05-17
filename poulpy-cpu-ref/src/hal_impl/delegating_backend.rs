use poulpy_core::{
    layouts::{GLWEBackendMut, GLWEBackendRef, GLWEInfos, GLWEToBackendMut, GLWEToBackendRef, LWEInfos},
    oep::{GLWEMulXpMinusOneImpl, GLWERotateImpl},
};
use poulpy_hal::{
    api::{
        VecZnxDftApply, VecZnxDftZero, VecZnxRotateAssignBackend, VecZnxRotateAssignTmpBytes, VecZnxRotateBackend,
        VecZnxZeroBackend, VmpApplyDftToDft,
    },
    layouts::{
        Backend, Module, NoiseInfos, ScratchArena, VecZnxBackendMut, VecZnxBackendRef, VecZnxDftToBackendMut,
        VecZnxDftToBackendRef, ZnxInfos,
    },
    oep::{HalConvolutionImpl, HalModuleImpl, HalSvpImpl, HalVecZnxBigImpl, HalVecZnxDftImpl, HalVecZnxImpl, HalVmpImpl},
};

use crate::{
    FFT64Ref,
    hal_defaults::{
        FFT64ConvolutionDefaults, FFT64ModuleDefaults, FFT64SvpDefaults, FFT64VecZnxBigDefaults, FFT64VecZnxDftDefaults,
        FFT64VmpDefaults, HalVecZnxDefaults,
    },
    reference::{
        fft64::{
            convolution::I64Ops,
            reim::{ReimArith, ReimFFTExecute, ReimFFTTable, ReimIFFTTable},
            reim4::{Reim4BlkMatVec, Reim4Convolution},
        },
        vec_znx::{vec_znx_mul_xp_minus_one, vec_znx_mul_xp_minus_one_assign},
        znx::{
            ZnxAdd, ZnxAddAssign, ZnxAutomorphism, ZnxCopy, ZnxExtractDigitAddMul, ZnxMulAddPowerOfTwo, ZnxMulPowerOfTwo,
            ZnxMulPowerOfTwoAssign, ZnxNegate, ZnxNegateAssign, ZnxNormalizeDigit, ZnxNormalizeFinalStep,
            ZnxNormalizeFinalStepAssign, ZnxNormalizeFinalStepSub, ZnxNormalizeFirstStep, ZnxNormalizeFirstStepAssign,
            ZnxNormalizeFirstStepCarryOnly, ZnxNormalizeMiddleStep, ZnxNormalizeMiddleStepAssign,
            ZnxNormalizeMiddleStepCarryOnly, ZnxNormalizeMiddleStepSub, ZnxRotate, ZnxSub, ZnxSubAssign, ZnxSubNegateAssign,
            ZnxSwitchRing, ZnxZero,
        },
    },
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct DelegatingFFT64Ref;

poulpy_hal::impl_backend_from!(DelegatingFFT64Ref, FFT64Ref);

macro_rules! impl_forward_znx_trait {
    ($trait_name:ident, $method:ident($($arg:ident : $ty:ty),*)) => {
        impl $trait_name for DelegatingFFT64Ref {
            #[inline(always)]
            fn $method($($arg : $ty),*) {
                <FFT64Ref as $trait_name>::$method($($arg),*)
            }
        }
    };
}

macro_rules! impl_forward_znx_const_trait {
    ($trait_name:ident, $method:ident($($arg:ident : $ty:ty),*)) => {
        impl $trait_name for DelegatingFFT64Ref {
            #[inline(always)]
            fn $method<const OVERWRITE: bool>($($arg : $ty),*) {
                <FFT64Ref as $trait_name>::$method::<OVERWRITE>($($arg),*)
            }
        }
    };
}

impl_forward_znx_trait!(ZnxAdd, znx_add(res: &mut [i64], a: &[i64], b: &[i64]));
impl_forward_znx_trait!(ZnxAddAssign, znx_add_assign(res: &mut [i64], a: &[i64]));
impl_forward_znx_trait!(ZnxSub, znx_sub(res: &mut [i64], a: &[i64], b: &[i64]));
impl_forward_znx_trait!(ZnxSubAssign, znx_sub_assign(res: &mut [i64], a: &[i64]));
impl_forward_znx_trait!(ZnxSubNegateAssign, znx_sub_negate_assign(res: &mut [i64], a: &[i64]));
impl_forward_znx_trait!(ZnxMulAddPowerOfTwo, znx_muladd_power_of_two(k: i64, res: &mut [i64], a: &[i64]));
impl_forward_znx_trait!(ZnxMulPowerOfTwo, znx_mul_power_of_two(k: i64, res: &mut [i64], a: &[i64]));
impl_forward_znx_trait!(ZnxMulPowerOfTwoAssign, znx_mul_power_of_two_assign(k: i64, res: &mut [i64]));
impl_forward_znx_trait!(ZnxAutomorphism, znx_automorphism(p: i64, res: &mut [i64], a: &[i64]));
impl_forward_znx_trait!(ZnxCopy, znx_copy(res: &mut [i64], a: &[i64]));
impl_forward_znx_trait!(ZnxNegate, znx_negate(res: &mut [i64], src: &[i64]));
impl_forward_znx_trait!(ZnxNegateAssign, znx_negate_assign(res: &mut [i64]));
impl_forward_znx_trait!(ZnxRotate, znx_rotate(p: i64, res: &mut [i64], src: &[i64]));
impl_forward_znx_trait!(ZnxZero, znx_zero(res: &mut [i64]));
impl_forward_znx_trait!(ZnxSwitchRing, znx_switch_ring(res: &mut [i64], a: &[i64]));
impl_forward_znx_const_trait!(
    ZnxNormalizeFirstStep,
    znx_normalize_first_step(base2k: usize, lsh: usize, x: &mut [i64], a: &[i64], carry: &mut [i64])
);
impl_forward_znx_const_trait!(
    ZnxNormalizeMiddleStep,
    znx_normalize_middle_step(base2k: usize, lsh: usize, x: &mut [i64], a: &[i64], carry: &mut [i64])
);
impl_forward_znx_const_trait!(
    ZnxNormalizeFinalStep,
    znx_normalize_final_step(base2k: usize, lsh: usize, x: &mut [i64], a: &[i64], carry: &mut [i64])
);
impl_forward_znx_trait!(
    ZnxNormalizeFirstStepCarryOnly,
    znx_normalize_first_step_carry_only(base2k: usize, lsh: usize, x: &[i64], carry: &mut [i64])
);
impl_forward_znx_trait!(
    ZnxNormalizeFirstStepAssign,
    znx_normalize_first_step_assign(base2k: usize, lsh: usize, x: &mut [i64], carry: &mut [i64])
);
impl_forward_znx_trait!(
    ZnxNormalizeMiddleStepCarryOnly,
    znx_normalize_middle_step_carry_only(base2k: usize, lsh: usize, x: &[i64], carry: &mut [i64])
);
impl_forward_znx_trait!(
    ZnxNormalizeMiddleStepAssign,
    znx_normalize_middle_step_assign(base2k: usize, lsh: usize, x: &mut [i64], carry: &mut [i64])
);
impl_forward_znx_trait!(
    ZnxNormalizeMiddleStepSub,
    znx_normalize_middle_step_sub(base2k: usize, lsh: usize, x: &mut [i64], a: &[i64], carry: &mut [i64])
);
impl_forward_znx_trait!(
    ZnxNormalizeFinalStepSub,
    znx_normalize_final_step_sub(base2k: usize, lsh: usize, x: &mut [i64], a: &[i64], carry: &mut [i64])
);
impl_forward_znx_trait!(
    ZnxNormalizeFinalStepAssign,
    znx_normalize_final_step_assign(base2k: usize, lsh: usize, x: &mut [i64], carry: &mut [i64])
);
impl_forward_znx_trait!(
    ZnxExtractDigitAddMul,
    znx_extract_digit_addmul(base2k: usize, lsh: usize, res: &mut [i64], src: &mut [i64])
);
impl_forward_znx_trait!(ZnxNormalizeDigit, znx_normalize_digit(base2k: usize, res: &mut [i64], src: &mut [i64]));

impl ReimFFTExecute<ReimFFTTable<f64>, f64> for DelegatingFFT64Ref {
    #[inline(always)]
    fn reim_dft_execute(table: &ReimFFTTable<f64>, data: &mut [f64]) {
        <FFT64Ref as ReimFFTExecute<ReimFFTTable<f64>, f64>>::reim_dft_execute(table, data)
    }
}

impl ReimFFTExecute<ReimIFFTTable<f64>, f64> for DelegatingFFT64Ref {
    #[inline(always)]
    fn reim_dft_execute(table: &ReimIFFTTable<f64>, data: &mut [f64]) {
        <FFT64Ref as ReimFFTExecute<ReimIFFTTable<f64>, f64>>::reim_dft_execute(table, data)
    }
}

impl ReimArith for DelegatingFFT64Ref {}
impl Reim4BlkMatVec for DelegatingFFT64Ref {}
impl Reim4Convolution for DelegatingFFT64Ref {}
impl I64Ops for DelegatingFFT64Ref {}

unsafe impl HalVecZnxImpl<DelegatingFFT64Ref> for DelegatingFFT64Ref {
    crate::hal_impl_vec_znx!();
}

unsafe impl HalModuleImpl<DelegatingFFT64Ref> for DelegatingFFT64Ref {
    crate::hal_impl_module!(FFT64ModuleDefaults);
}

unsafe impl HalVmpImpl<DelegatingFFT64Ref> for DelegatingFFT64Ref {
    crate::hal_impl_vmp!(FFT64VmpDefaults);
}

unsafe impl HalConvolutionImpl<DelegatingFFT64Ref> for DelegatingFFT64Ref {
    crate::hal_impl_convolution!(FFT64ConvolutionDefaults);
}

unsafe impl HalVecZnxBigImpl<DelegatingFFT64Ref> for DelegatingFFT64Ref {
    crate::hal_impl_vec_znx_big!(FFT64VecZnxBigDefaults);
}

unsafe impl HalSvpImpl<DelegatingFFT64Ref> for DelegatingFFT64Ref {
    crate::hal_impl_svp!(FFT64SvpDefaults);
}

unsafe impl HalVecZnxDftImpl<DelegatingFFT64Ref> for DelegatingFFT64Ref {
    crate::hal_impl_vec_znx_dft!(FFT64VecZnxDftDefaults);
}

unsafe impl GLWEMulXpMinusOneImpl<DelegatingFFT64Ref> for DelegatingFFT64Ref {
    fn glwe_mul_xp_minus_one<R, A>(module: &Module<DelegatingFFT64Ref>, k: i64, res: &mut R, a: &A)
    where
        R: GLWEToBackendMut<DelegatingFFT64Ref>,
        A: GLWEToBackendRef<DelegatingFFT64Ref>,
    {
        let res: &mut GLWEBackendMut<'_, DelegatingFFT64Ref> = &mut res.to_backend_mut();
        let a: &GLWEBackendRef<'_, DelegatingFFT64Ref> = &a.to_backend_ref();

        assert_eq!(res.n(), module.n() as u32);
        assert_eq!(a.n(), module.n() as u32);
        assert_eq!(res.rank(), a.rank());

        for i in 0..res.rank().as_usize() + 1 {
            vec_znx_mul_xp_minus_one::<DelegatingFFT64Ref>(k, res.data_mut(), i, a.data(), i);
        }
    }

    fn glwe_mul_xp_minus_one_assign<'s, R>(
        module: &Module<DelegatingFFT64Ref>,
        k: i64,
        res: &mut R,
        _scratch: &mut ScratchArena<'s, DelegatingFFT64Ref>,
    ) where
        R: GLWEToBackendMut<DelegatingFFT64Ref>,
    {
        let res: &mut GLWEBackendMut<'_, DelegatingFFT64Ref> = &mut res.to_backend_mut();

        assert_eq!(res.n(), module.n() as u32);

        let mut tmp = vec![0i64; module.n()];
        for i in 0..res.rank().as_usize() + 1 {
            vec_znx_mul_xp_minus_one_assign::<DelegatingFFT64Ref>(k, res.data_mut(), i, &mut tmp);
        }
    }
}

unsafe impl GLWERotateImpl<DelegatingFFT64Ref> for DelegatingFFT64Ref {
    fn glwe_rotate_tmp_bytes(module: &Module<DelegatingFFT64Ref>) -> usize {
        module.vec_znx_rotate_assign_tmp_bytes()
    }

    fn glwe_rotate<R, A>(module: &Module<DelegatingFFT64Ref>, k: i64, res: &mut R, a: &A)
    where
        R: GLWEToBackendMut<DelegatingFFT64Ref>,
        A: GLWEToBackendRef<DelegatingFFT64Ref>,
    {
        let mut res = res.to_backend_mut();
        let a = a.to_backend_ref();

        assert_eq!(a.n(), module.n() as u32);
        assert_eq!(res.n(), module.n() as u32);
        assert!(res.rank() == a.rank() || a.rank() == 0);

        let res_cols = (res.rank() + 1).into();
        let a_cols = (a.rank() + 1).into();

        for i in 0..a_cols {
            module.vec_znx_rotate_backend(k, res.data_mut(), i, a.data(), i);
        }
        for i in a_cols..res_cols {
            module.vec_znx_zero_backend(res.data_mut(), i);
        }
    }

    fn glwe_rotate_assign<'s, R>(
        module: &Module<DelegatingFFT64Ref>,
        k: i64,
        res: &mut R,
        scratch: &mut ScratchArena<'s, DelegatingFFT64Ref>,
    ) where
        R: GLWEToBackendMut<DelegatingFFT64Ref>,
    {
        let mut res = res.to_backend_mut();

        assert_eq!(res.n(), module.n() as u32);

        for i in 0..(res.rank() + 1).into() {
            let mut scratch_iter = scratch.borrow();
            module.vec_znx_rotate_assign_backend(k, res.data_mut(), i, &mut scratch_iter);
        }
    }
}
