use poulpy_hal::{
    layouts::{Backend, Module, ScalarZnxToBackendRef, ScratchArena},
    source::Source,
};

use crate::{
    GetDistribution, GetDistributionMut,
    api::{
        EncryptionInfos, GGLWECompressedEncryptSk, GGLWEEncryptSk, GGLWEToGGSWKeyCompressedEncryptSk, GGLWEToGGSWKeyEncryptSk,
        GGSWCompressedEncryptSk, GGSWEncryptSk, GLWEAutomorphismKeyCompressedEncryptSk, GLWEAutomorphismKeyEncryptPk,
        GLWEAutomorphismKeyEncryptSk, GLWECompressedEncryptSk, GLWEEncryptPk, GLWEEncryptSk, GLWEPublicKeyGenerate,
        GLWESwitchingKeyCompressedEncryptSk, GLWESwitchingKeyEncryptPk, GLWESwitchingKeyEncryptSk,
        GLWETensorKeyCompressedEncryptSk, GLWETensorKeyEncryptSk, GLWEToLWESwitchingKeyEncryptSk, LWEEncryptSk,
        LWESwitchingKeyEncrypt, LWEToGLWESwitchingKeyEncryptSk,
    },
    encryption::{
        GGLWECompressedEncryptSkDefault, GGLWEEncryptSkDefault, GGLWEToGGSWKeyCompressedEncryptSkDefault,
        GGLWEToGGSWKeyEncryptSkDefault, GGSWCompressedEncryptSkDefault, GGSWEncryptSkDefault,
        GLWEAutomorphismKeyCompressedEncryptSkDefault, GLWEAutomorphismKeyEncryptPkDefault, GLWEAutomorphismKeyEncryptSkDefault,
        GLWECompressedEncryptSkDefault, GLWEEncryptPkDefault, GLWEEncryptSkDefault, GLWEPublicKeyGenerateDefault,
        GLWESwitchingKeyCompressedEncryptSkDefault, GLWESwitchingKeyEncryptPkDefault, GLWESwitchingKeyEncryptSkDefault,
        GLWETensorKeyCompressedEncryptSkDefault, GLWETensorKeyEncryptSkDefault, GLWEToLWESwitchingKeyEncryptSkDefault,
        LWEEncryptSkDefault, LWESwitchingKeyEncryptDefault, LWEToGLWESwitchingKeyEncryptSkDefault,
    },
    layouts::{
        GGLWECompressedSeedMut, GGLWECompressedToBackendMut, GGLWEInfos, GGLWEToBackendMut, GGLWEToGGSWKeyCompressedToBackendMut,
        GGLWEToGGSWKeyToBackendMut, GGSWCompressedSeedMut, GGSWCompressedToBackendMut, GGSWInfos, GGSWToBackendMut,
        GLWECompressedSeedMut, GLWECompressedToBackendMut, GLWEInfos, GLWESecretToBackendRef, GLWESwitchingKeyDegreesMut,
        GLWEToBackendMut, GLWEToBackendRef, LWEInfos, LWEPlaintextToBackendRef, LWESecretToBackendRef, LWEToBackendMut,
        SetGaloisElement,
        prepared::{GLWEPreparedToBackendRef, GLWESecretPreparedToBackendRef},
    },
};

macro_rules! impl_encryption_delegate {
    ($trait:ty, $default:path, $($body:item),+ $(,)?) => {
        impl<BE> $trait for Module<BE>
        where
            BE: Backend,
            Module<BE>: $default,
        {
            $($body)+
        }
    };
}

impl_encryption_delegate!(
    LWEEncryptSk<BE>,
    LWEEncryptSkDefault<BE>,
    fn lwe_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: LWEInfos,
    {
        LWEEncryptSkDefault::lwe_encrypt_sk_tmp_bytes(self, infos)
    },
    fn lwe_encrypt_sk<'s, R, P, S, E>(
        &self,
        res: &mut R,
        pt: &P,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: LWEToBackendMut<BE>,
        P: LWEPlaintextToBackendRef<BE>,
        S: LWESecretToBackendRef<BE>,
        E: EncryptionInfos,
        for<'a> ScratchArena<'a, BE>: crate::ScratchArenaTakeCore<'a, BE>,
    {
        LWEEncryptSkDefault::lwe_encrypt_sk(self, res, pt, sk, enc_infos, source_xe, source_xa, scratch)
    }
);

impl_encryption_delegate!(
    GLWEEncryptSk<BE>,
    GLWEEncryptSkDefault<BE>,
    fn glwe_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GLWEInfos,
    {
        GLWEEncryptSkDefault::glwe_encrypt_sk_tmp_bytes(self, infos)
    },
    fn glwe_encrypt_sk<'s, R, P, S, E>(
        &self,
        res: &mut R,
        pt: &P,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE>,
        P: GLWEToBackendRef<BE>,
        E: EncryptionInfos,
        S: GLWESecretPreparedToBackendRef<BE>,
        BE: 's,
        for<'a> ScratchArena<'a, BE>: crate::ScratchArenaTakeCore<'a, BE>,
    {
        GLWEEncryptSkDefault::glwe_encrypt_sk(self, res, pt, sk, enc_infos, source_xe, source_xa, scratch)
    },
    fn glwe_encrypt_zero_sk<'s, R, E, S>(
        &self,
        res: &mut R,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE>,
        E: EncryptionInfos,
        S: GLWESecretPreparedToBackendRef<BE>,
        BE: 's,
        for<'a> ScratchArena<'a, BE>: crate::ScratchArenaTakeCore<'a, BE>,
    {
        GLWEEncryptSkDefault::glwe_encrypt_zero_sk(self, res, sk, enc_infos, source_xe, source_xa, scratch)
    }
);

impl_encryption_delegate!(
    GLWEEncryptPk<BE>,
    GLWEEncryptPkDefault<BE>,
    fn glwe_encrypt_pk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GLWEInfos,
    {
        GLWEEncryptPkDefault::glwe_encrypt_pk_tmp_bytes(self, infos)
    },
    fn glwe_encrypt_pk<'s, R, P, K, E>(
        &self,
        res: &mut R,
        pt: &P,
        pk: &K,
        enc_infos: &E,
        source_xu: &mut Source,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        P: GLWEToBackendRef<BE> + GLWEInfos,
        E: EncryptionInfos,
        K: GLWEPreparedToBackendRef<BE> + GetDistribution + GLWEInfos,
        BE: 's,
        for<'a> ScratchArena<'a, BE>: crate::ScratchArenaTakeCore<'a, BE>,
    {
        GLWEEncryptPkDefault::glwe_encrypt_pk(self, res, pt, pk, enc_infos, source_xu, source_xe, scratch)
    },
    fn glwe_encrypt_zero_pk<'s, R, K, E>(
        &self,
        res: &mut R,
        pk: &K,
        enc_infos: &E,
        source_xu: &mut Source,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWEToBackendMut<BE> + GLWEInfos,
        E: EncryptionInfos,
        K: GLWEPreparedToBackendRef<BE> + GetDistribution + GLWEInfos,
        BE: 's,
        for<'a> ScratchArena<'a, BE>: crate::ScratchArenaTakeCore<'a, BE>,
    {
        GLWEEncryptPkDefault::glwe_encrypt_zero_pk(self, res, pk, enc_infos, source_xu, source_xe, scratch)
    }
);

impl_encryption_delegate!(
    GLWEPublicKeyGenerate<BE>,
    GLWEPublicKeyGenerateDefault<BE>,
    fn glwe_public_key_generate<R, S, E>(
        &self,
        res: &mut R,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
    ) where
        R: GLWEToBackendMut<BE> + GetDistributionMut + GLWEInfos,
        E: EncryptionInfos,
        S: GLWESecretPreparedToBackendRef<BE> + GetDistribution,
    {
        GLWEPublicKeyGenerateDefault::glwe_public_key_generate(self, res, sk, enc_infos, source_xe, source_xa)
    }
);

impl_encryption_delegate!(
    GGLWEEncryptSk<BE>,
    GGLWEEncryptSkDefault<BE>,
    fn gglwe_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GGLWEEncryptSkDefault::gglwe_encrypt_sk_tmp_bytes(self, infos)
    },
    fn gglwe_encrypt_sk<'s, R, P, S, E>(
        &self,
        res: &mut R,
        pt: &P,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGLWEToBackendMut<BE>,
        P: ScalarZnxToBackendRef<BE>,
        E: EncryptionInfos,
        S: GLWESecretPreparedToBackendRef<BE>,
        for<'a> ScratchArena<'a, BE>: crate::ScratchArenaTakeCore<'a, BE>,
    {
        GGLWEEncryptSkDefault::gglwe_encrypt_sk(self, res, pt, sk, enc_infos, source_xe, source_xa, scratch)
    }
);

impl_encryption_delegate!(
    GGSWEncryptSk<BE>,
    GGSWEncryptSkDefault<BE>,
    fn ggsw_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGSWInfos,
    {
        GGSWEncryptSkDefault::ggsw_encrypt_sk_tmp_bytes(self, infos)
    },
    fn ggsw_encrypt_sk<'s, R, P, S, E>(
        &self,
        res: &mut R,
        pt: &P,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGSWToBackendMut<BE>,
        P: ScalarZnxToBackendRef<BE>,
        E: EncryptionInfos,
        S: GLWESecretPreparedToBackendRef<BE>,
        for<'a> ScratchArena<'a, BE>: crate::ScratchArenaTakeCore<'a, BE>,
    {
        GGSWEncryptSkDefault::ggsw_encrypt_sk(self, res, pt, sk, enc_infos, source_xe, source_xa, scratch)
    }
);

impl_encryption_delegate!(
    GGLWEToGGSWKeyEncryptSk<BE>,
    GGLWEToGGSWKeyEncryptSkDefault<BE>,
    fn gglwe_to_ggsw_key_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GGLWEToGGSWKeyEncryptSkDefault::gglwe_to_ggsw_key_encrypt_sk_tmp_bytes(self, infos)
    },
    fn gglwe_to_ggsw_key_encrypt_sk<R, S, E>(
        &self,
        res: &mut R,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWEToGGSWKeyToBackendMut<BE>,
        E: EncryptionInfos,
        S: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos,
    {
        GGLWEToGGSWKeyEncryptSkDefault::gglwe_to_ggsw_key_encrypt_sk(self, res, sk, enc_infos, source_xe, source_xa, scratch)
    }
);

impl_encryption_delegate!(
    GLWESwitchingKeyEncryptSk<BE>,
    GLWESwitchingKeyEncryptSkDefault<BE>,
    fn glwe_switching_key_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GLWESwitchingKeyEncryptSkDefault::glwe_switching_key_encrypt_sk_tmp_bytes(self, infos)
    },
    fn glwe_switching_key_encrypt_sk<R, S1, S2, E>(
        &self,
        res: &mut R,
        sk_in: &S1,
        sk_out: &S2,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + GLWESwitchingKeyDegreesMut + GGLWEInfos,
        E: EncryptionInfos,
        S1: GLWESecretToBackendRef<BE> + GLWEInfos,
        S2: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos,
    {
        GLWESwitchingKeyEncryptSkDefault::glwe_switching_key_encrypt_sk(
            self, res, sk_in, sk_out, enc_infos, source_xe, source_xa, scratch,
        )
    }
);

impl_encryption_delegate!(
    GLWESwitchingKeyEncryptPk<BE>,
    GLWESwitchingKeyEncryptPkDefault<BE>,
    fn glwe_switching_key_encrypt_pk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GLWESwitchingKeyEncryptPkDefault::glwe_switching_key_encrypt_pk_tmp_bytes(self, infos)
    }
);

impl_encryption_delegate!(
    GLWETensorKeyEncryptSk<BE>,
    GLWETensorKeyEncryptSkDefault<BE>,
    fn glwe_tensor_key_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GLWETensorKeyEncryptSkDefault::glwe_tensor_key_encrypt_sk_tmp_bytes(self, infos)
    },
    fn glwe_tensor_key_encrypt_sk<R, S, E>(
        &self,
        res: &mut R,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + GGLWEInfos,
        E: EncryptionInfos,
        S: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos,
    {
        GLWETensorKeyEncryptSkDefault::glwe_tensor_key_encrypt_sk(self, res, sk, enc_infos, source_xe, source_xa, scratch)
    }
);

impl_encryption_delegate!(
    GLWEToLWESwitchingKeyEncryptSk<BE>,
    GLWEToLWESwitchingKeyEncryptSkDefault<BE>,
    fn glwe_to_lwe_key_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GLWEToLWESwitchingKeyEncryptSkDefault::glwe_to_lwe_key_encrypt_sk_tmp_bytes(self, infos)
    },
    fn glwe_to_lwe_key_encrypt_sk<R, S1, S2, E>(
        &self,
        res: &mut R,
        sk_lwe: &S1,
        sk_glwe: &S2,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        S1: LWESecretToBackendRef<BE>,
        S2: GLWESecretToBackendRef<BE>,
        E: EncryptionInfos,
        R: GGLWEToBackendMut<BE> + GGLWEInfos,
    {
        GLWEToLWESwitchingKeyEncryptSkDefault::glwe_to_lwe_key_encrypt_sk(
            self, res, sk_lwe, sk_glwe, enc_infos, source_xe, source_xa, scratch,
        )
    }
);

impl_encryption_delegate!(
    LWESwitchingKeyEncrypt<BE>,
    LWESwitchingKeyEncryptDefault<BE>,
    fn lwe_switching_key_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        LWESwitchingKeyEncryptDefault::lwe_switching_key_encrypt_sk_tmp_bytes(self, infos)
    },
    fn lwe_switching_key_encrypt_sk<R, S1, S2, E>(
        &self,
        res: &mut R,
        sk_lwe_in: &S1,
        sk_lwe_out: &S2,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + GLWESwitchingKeyDegreesMut + GGLWEInfos,
        E: EncryptionInfos,
        S1: LWESecretToBackendRef<BE>,
        S2: LWESecretToBackendRef<BE>,
    {
        LWESwitchingKeyEncryptDefault::lwe_switching_key_encrypt_sk(
            self, res, sk_lwe_in, sk_lwe_out, enc_infos, source_xe, source_xa, scratch,
        )
    }
);

impl_encryption_delegate!(
    LWEToGLWESwitchingKeyEncryptSk<BE>,
    LWEToGLWESwitchingKeyEncryptSkDefault<BE>,
    fn lwe_to_glwe_key_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        LWEToGLWESwitchingKeyEncryptSkDefault::lwe_to_glwe_key_encrypt_sk_tmp_bytes(self, infos)
    },
    fn lwe_to_glwe_key_encrypt_sk<R, S1, S2, E>(
        &self,
        res: &mut R,
        sk_lwe: &S1,
        sk_glwe: &S2,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        S1: LWESecretToBackendRef<BE>,
        S2: GLWESecretPreparedToBackendRef<BE>,
        E: EncryptionInfos,
        R: GGLWEToBackendMut<BE> + GGLWEInfos,
    {
        LWEToGLWESwitchingKeyEncryptSkDefault::lwe_to_glwe_key_encrypt_sk(
            self, res, sk_lwe, sk_glwe, enc_infos, source_xe, source_xa, scratch,
        )
    }
);

impl_encryption_delegate!(
    GLWEAutomorphismKeyEncryptSk<BE>,
    GLWEAutomorphismKeyEncryptSkDefault<BE>,
    fn glwe_automorphism_key_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GLWEAutomorphismKeyEncryptSkDefault::glwe_automorphism_key_encrypt_sk_tmp_bytes(self, infos)
    },
    fn glwe_automorphism_key_encrypt_sk<R, S, E>(
        &self,
        res: &mut R,
        p: i64,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + SetGaloisElement + GGLWEInfos,
        E: EncryptionInfos,
        S: GLWESecretToBackendRef<BE> + GLWEInfos,
    {
        GLWEAutomorphismKeyEncryptSkDefault::glwe_automorphism_key_encrypt_sk(
            self, res, p, sk, enc_infos, source_xe, source_xa, scratch,
        )
    }
);

impl_encryption_delegate!(
    GLWEAutomorphismKeyEncryptPk<BE>,
    GLWEAutomorphismKeyEncryptPkDefault<BE>,
    fn glwe_automorphism_key_encrypt_pk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GLWEAutomorphismKeyEncryptPkDefault::glwe_automorphism_key_encrypt_pk_tmp_bytes(self, infos)
    }
);

impl_encryption_delegate!(
    GLWECompressedEncryptSk<BE>,
    GLWECompressedEncryptSkDefault<BE>,
    fn glwe_compressed_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GLWEInfos,
    {
        GLWECompressedEncryptSkDefault::glwe_compressed_encrypt_sk_tmp_bytes(self, infos)
    },
    fn glwe_compressed_encrypt_sk<'s, R, P, S, E>(
        &self,
        res: &'s mut R,
        pt: &P,
        sk: &S,
        seed_xa: [u8; 32],
        enc_infos: &E,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GLWECompressedToBackendMut<BE> + GLWECompressedSeedMut,
        P: GLWEToBackendRef<BE>,
        E: EncryptionInfos,
        S: GLWESecretPreparedToBackendRef<BE>,
        BE: 's,
        ScratchArena<'s, BE>: crate::ScratchArenaTakeCore<'s, BE>,
    {
        GLWECompressedEncryptSkDefault::glwe_compressed_encrypt_sk(self, res, pt, sk, seed_xa, enc_infos, source_xe, scratch)
    }
);

impl_encryption_delegate!(
    GGLWECompressedEncryptSk<BE>,
    GGLWECompressedEncryptSkDefault<BE>,
    fn gglwe_compressed_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GGLWECompressedEncryptSkDefault::gglwe_compressed_encrypt_sk_tmp_bytes(self, infos)
    },
    fn gglwe_compressed_encrypt_sk<'s, R, P, S, E>(
        &self,
        res: &mut R,
        pt: &P,
        sk: &S,
        seed: [u8; 32],
        enc_infos: &E,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGLWECompressedToBackendMut<BE> + GGLWECompressedSeedMut,
        P: ScalarZnxToBackendRef<BE>,
        E: EncryptionInfos,
        S: GLWESecretPreparedToBackendRef<BE>,
        for<'a> ScratchArena<'a, BE>: crate::ScratchArenaTakeCore<'a, BE>,
    {
        GGLWECompressedEncryptSkDefault::gglwe_compressed_encrypt_sk(self, res, pt, sk, seed, enc_infos, source_xe, scratch)
    }
);

impl_encryption_delegate!(
    GGSWCompressedEncryptSk<BE>,
    GGSWCompressedEncryptSkDefault<BE>,
    fn ggsw_compressed_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGSWInfos,
    {
        GGSWCompressedEncryptSkDefault::ggsw_compressed_encrypt_sk_tmp_bytes(self, infos)
    },
    fn ggsw_compressed_encrypt_sk<'s, R, P, S, E>(
        &self,
        res: &mut R,
        pt: &P,
        sk: &S,
        seed_xa: [u8; 32],
        enc_infos: &E,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'s, BE>,
    ) where
        R: GGSWCompressedToBackendMut<BE> + GGSWCompressedSeedMut + GGSWInfos,
        P: ScalarZnxToBackendRef<BE>,
        E: EncryptionInfos,
        S: GLWESecretPreparedToBackendRef<BE>,
        for<'a> ScratchArena<'a, BE>: crate::ScratchArenaTakeCore<'a, BE>,
    {
        GGSWCompressedEncryptSkDefault::ggsw_compressed_encrypt_sk(self, res, pt, sk, seed_xa, enc_infos, source_xe, scratch)
    }
);

impl_encryption_delegate!(
    GGLWEToGGSWKeyCompressedEncryptSk<BE>,
    GGLWEToGGSWKeyCompressedEncryptSkDefault<BE>,
    fn gglwe_to_ggsw_key_compressed_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GGLWEToGGSWKeyCompressedEncryptSkDefault::gglwe_to_ggsw_key_compressed_encrypt_sk_tmp_bytes(self, infos)
    },
    fn gglwe_to_ggsw_key_compressed_encrypt_sk<R, S, E>(
        &self,
        res: &mut R,
        sk: &S,
        seed_xa: [u8; 32],
        enc_infos: &E,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWEToGGSWKeyCompressedToBackendMut<BE> + GGLWEInfos,
        E: EncryptionInfos,
        S: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos,
    {
        GGLWEToGGSWKeyCompressedEncryptSkDefault::gglwe_to_ggsw_key_compressed_encrypt_sk(
            self, res, sk, seed_xa, enc_infos, source_xe, scratch,
        )
    }
);

impl_encryption_delegate!(
    GLWEAutomorphismKeyCompressedEncryptSk<BE>,
    GLWEAutomorphismKeyCompressedEncryptSkDefault<BE>,
    fn glwe_automorphism_key_compressed_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GLWEAutomorphismKeyCompressedEncryptSkDefault::glwe_automorphism_key_compressed_encrypt_sk_tmp_bytes(self, infos)
    },
    fn glwe_automorphism_key_compressed_encrypt_sk<R, S, E>(
        &self,
        res: &mut R,
        p: i64,
        sk: &S,
        seed_xa: [u8; 32],
        enc_infos: &E,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWECompressedToBackendMut<BE> + GGLWECompressedSeedMut + SetGaloisElement + GGLWEInfos,
        E: EncryptionInfos,
        S: GLWESecretToBackendRef<BE> + GLWEInfos,
    {
        GLWEAutomorphismKeyCompressedEncryptSkDefault::glwe_automorphism_key_compressed_encrypt_sk(
            self, res, p, sk, seed_xa, enc_infos, source_xe, scratch,
        )
    }
);

impl_encryption_delegate!(
    GLWESwitchingKeyCompressedEncryptSk<BE>,
    GLWESwitchingKeyCompressedEncryptSkDefault<BE>,
    fn glwe_switching_key_compressed_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GLWESwitchingKeyCompressedEncryptSkDefault::glwe_switching_key_compressed_encrypt_sk_tmp_bytes(self, infos)
    },
    fn glwe_switching_key_compressed_encrypt_sk<R, S1, S2, E>(
        &self,
        res: &mut R,
        sk_in: &S1,
        sk_out: &S2,
        seed_xa: [u8; 32],
        enc_infos: &E,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWECompressedToBackendMut<BE> + GGLWECompressedSeedMut + GLWESwitchingKeyDegreesMut + GGLWEInfos,
        E: EncryptionInfos,
        S1: GLWESecretToBackendRef<BE> + GLWEInfos,
        S2: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos,
    {
        GLWESwitchingKeyCompressedEncryptSkDefault::glwe_switching_key_compressed_encrypt_sk(
            self, res, sk_in, sk_out, seed_xa, enc_infos, source_xe, scratch,
        )
    }
);

impl_encryption_delegate!(
    GLWETensorKeyCompressedEncryptSk<BE>,
    GLWETensorKeyCompressedEncryptSkDefault<BE>,
    fn glwe_tensor_key_compressed_encrypt_sk_tmp_bytes<A>(&self, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        GLWETensorKeyCompressedEncryptSkDefault::glwe_tensor_key_compressed_encrypt_sk_tmp_bytes(self, infos)
    },
    fn glwe_tensor_key_compressed_encrypt_sk<R, S, E>(
        &self,
        res: &mut R,
        sk: &S,
        seed_xa: [u8; 32],
        enc_infos: &E,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWECompressedToBackendMut<BE> + GGLWEInfos + GGLWECompressedSeedMut,
        E: EncryptionInfos,
        S: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos,
    {
        GLWETensorKeyCompressedEncryptSkDefault::glwe_tensor_key_compressed_encrypt_sk(
            self, res, sk, seed_xa, enc_infos, source_xe, scratch,
        )
    }
);
