#![allow(clippy::too_many_arguments)]

use poulpy_hal::{
    layouts::{Backend, Module, ScalarZnxToBackendRef, ScratchArena},
    source::Source,
};

use crate::{
    EncryptionInfos, GetDistribution, GetDistributionMut, ScratchArenaTakeCore,
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

#[doc(hidden)]
pub trait EncryptionDefaults<BE: Backend>: Backend {
    fn lwe_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: LWEInfos;

    fn lwe_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>;

    fn glwe_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GLWEInfos;

    fn glwe_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>;

    fn glwe_encrypt_zero_sk_default<'s, R, E, S>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>;

    fn glwe_encrypt_pk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GLWEInfos;

    fn glwe_encrypt_pk_default<'s, R, P, K, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>;

    fn glwe_encrypt_zero_pk_default<'s, R, K, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>;

    fn glwe_public_key_generate_default<R, S, E>(
        module: &Module<BE>,
        res: &mut R,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
    ) where
        R: GLWEToBackendMut<BE> + GetDistributionMut + GLWEInfos,
        E: EncryptionInfos,
        S: GLWESecretPreparedToBackendRef<BE> + GetDistribution;

    fn gglwe_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn gglwe_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>;

    fn ggsw_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGSWInfos;

    fn ggsw_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>;

    fn gglwe_to_ggsw_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn gglwe_to_ggsw_key_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
        res: &mut R,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWEToGGSWKeyToBackendMut<BE>,
        E: EncryptionInfos,
        S: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos;

    fn glwe_switching_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn glwe_switching_key_encrypt_sk_default<R, S1, S2, E>(
        module: &Module<BE>,
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
        S2: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos;

    fn glwe_switching_key_encrypt_pk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn glwe_tensor_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn glwe_tensor_key_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
        res: &mut R,
        sk: &S,
        enc_infos: &E,
        source_xe: &mut Source,
        source_xa: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWEToBackendMut<BE> + GGLWEInfos,
        E: EncryptionInfos,
        S: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos;

    fn glwe_to_lwe_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn glwe_to_lwe_key_encrypt_sk_default<R, S1, S2, E>(
        module: &Module<BE>,
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
        R: GGLWEToBackendMut<BE> + GGLWEInfos;

    fn lwe_switching_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn lwe_switching_key_encrypt_sk_default<R, S1, S2, E>(
        module: &Module<BE>,
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
        S2: LWESecretToBackendRef<BE>;

    fn lwe_to_glwe_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn lwe_to_glwe_key_encrypt_sk_default<R, S1, S2, E>(
        module: &Module<BE>,
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
        R: GGLWEToBackendMut<BE> + GGLWEInfos;

    fn glwe_automorphism_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn glwe_automorphism_key_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
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
        S: GLWESecretToBackendRef<BE> + GLWEInfos;

    fn glwe_automorphism_key_encrypt_pk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn glwe_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GLWEInfos;

    fn glwe_compressed_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        ScratchArena<'s, BE>: ScratchArenaTakeCore<'s, BE>;

    fn gglwe_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn gglwe_compressed_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>;

    fn ggsw_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGSWInfos;

    fn ggsw_compressed_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>;

    fn gglwe_to_ggsw_key_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn gglwe_to_ggsw_key_compressed_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
        res: &mut R,
        sk: &S,
        seed_xa: [u8; 32],
        enc_infos: &E,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWEToGGSWKeyCompressedToBackendMut<BE> + GGLWEInfos,
        E: EncryptionInfos,
        S: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos;

    fn glwe_automorphism_key_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn glwe_automorphism_key_compressed_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
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
        S: GLWESecretToBackendRef<BE> + GLWEInfos;

    fn glwe_switching_key_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn glwe_switching_key_compressed_encrypt_sk_default<R, S1, S2, E>(
        module: &Module<BE>,
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
        S2: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos;

    fn glwe_tensor_key_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos;

    fn glwe_tensor_key_compressed_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
        res: &mut R,
        sk: &S,
        seed_xa: [u8; 32],
        enc_infos: &E,
        source_xe: &mut Source,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        R: GGLWECompressedToBackendMut<BE> + GGLWEInfos + GGLWECompressedSeedMut,
        E: EncryptionInfos,
        S: GLWESecretToBackendRef<BE> + GetDistribution + GLWEInfos;
}

impl<BE: Backend> EncryptionDefaults<BE> for BE
where
    Module<BE>: LWEEncryptSkDefault<BE>
        + GLWEEncryptSkDefault<BE>
        + GLWEEncryptPkDefault<BE>
        + GLWEPublicKeyGenerateDefault<BE>
        + GGLWEEncryptSkDefault<BE>
        + GGSWEncryptSkDefault<BE>
        + GGLWEToGGSWKeyEncryptSkDefault<BE>
        + GLWESwitchingKeyEncryptSkDefault<BE>
        + GLWESwitchingKeyEncryptPkDefault<BE>
        + GLWETensorKeyEncryptSkDefault<BE>
        + GLWEToLWESwitchingKeyEncryptSkDefault<BE>
        + LWESwitchingKeyEncryptDefault<BE>
        + LWEToGLWESwitchingKeyEncryptSkDefault<BE>
        + GLWEAutomorphismKeyEncryptSkDefault<BE>
        + GLWEAutomorphismKeyEncryptPkDefault<BE>
        + GLWECompressedEncryptSkDefault<BE>
        + GGLWECompressedEncryptSkDefault<BE>
        + GGSWCompressedEncryptSkDefault<BE>
        + GGLWEToGGSWKeyCompressedEncryptSkDefault<BE>
        + GLWEAutomorphismKeyCompressedEncryptSkDefault<BE>
        + GLWESwitchingKeyCompressedEncryptSkDefault<BE>
        + GLWETensorKeyCompressedEncryptSkDefault<BE>,
    for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
{
    fn lwe_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: LWEInfos,
    {
        module.lwe_encrypt_sk_tmp_bytes(infos)
    }

    fn lwe_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
    {
        module.lwe_encrypt_sk(res, pt, sk, enc_infos, source_xe, source_xa, scratch)
    }

    fn glwe_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GLWEInfos,
    {
        module.glwe_encrypt_sk_tmp_bytes(infos)
    }

    fn glwe_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
    {
        module.glwe_encrypt_sk(res, pt, sk, enc_infos, source_xe, source_xa, scratch)
    }

    fn glwe_encrypt_zero_sk_default<'s, R, E, S>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
    {
        module.glwe_encrypt_zero_sk(res, sk, enc_infos, source_xe, source_xa, scratch)
    }

    fn glwe_encrypt_pk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GLWEInfos,
    {
        module.glwe_encrypt_pk_tmp_bytes(infos)
    }

    fn glwe_encrypt_pk_default<'s, R, P, K, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
    {
        module.glwe_encrypt_pk(res, pt, pk, enc_infos, source_xu, source_xe, scratch)
    }

    fn glwe_encrypt_zero_pk_default<'s, R, K, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
    {
        module.glwe_encrypt_zero_pk(res, pk, enc_infos, source_xu, source_xe, scratch)
    }

    fn glwe_public_key_generate_default<R, S, E>(
        module: &Module<BE>,
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
        module.glwe_public_key_generate(res, sk, enc_infos, source_xe, source_xa)
    }

    fn gglwe_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.gglwe_encrypt_sk_tmp_bytes(infos)
    }

    fn gglwe_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
    {
        module.gglwe_encrypt_sk(res, pt, sk, enc_infos, source_xe, source_xa, scratch)
    }

    fn ggsw_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGSWInfos,
    {
        module.ggsw_encrypt_sk_tmp_bytes(infos)
    }

    fn ggsw_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
    {
        module.ggsw_encrypt_sk(res, pt, sk, enc_infos, source_xe, source_xa, scratch)
    }

    fn gglwe_to_ggsw_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.gglwe_to_ggsw_key_encrypt_sk_tmp_bytes(infos)
    }

    fn gglwe_to_ggsw_key_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
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
        module.gglwe_to_ggsw_key_encrypt_sk(res, sk, enc_infos, source_xe, source_xa, scratch)
    }

    fn glwe_switching_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.glwe_switching_key_encrypt_sk_tmp_bytes(infos)
    }

    fn glwe_switching_key_encrypt_sk_default<R, S1, S2, E>(
        module: &Module<BE>,
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
        module.glwe_switching_key_encrypt_sk(res, sk_in, sk_out, enc_infos, source_xe, source_xa, scratch)
    }

    fn glwe_switching_key_encrypt_pk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.glwe_switching_key_encrypt_pk_tmp_bytes(infos)
    }

    fn glwe_tensor_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.glwe_tensor_key_encrypt_sk_tmp_bytes(infos)
    }

    fn glwe_tensor_key_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
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
        module.glwe_tensor_key_encrypt_sk(res, sk, enc_infos, source_xe, source_xa, scratch)
    }

    fn glwe_to_lwe_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.glwe_to_lwe_key_encrypt_sk_tmp_bytes(infos)
    }

    fn glwe_to_lwe_key_encrypt_sk_default<R, S1, S2, E>(
        module: &Module<BE>,
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
        module.glwe_to_lwe_key_encrypt_sk(res, sk_lwe, sk_glwe, enc_infos, source_xe, source_xa, scratch)
    }

    fn lwe_switching_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.lwe_switching_key_encrypt_sk_tmp_bytes(infos)
    }

    fn lwe_switching_key_encrypt_sk_default<R, S1, S2, E>(
        module: &Module<BE>,
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
        module.lwe_switching_key_encrypt_sk(res, sk_lwe_in, sk_lwe_out, enc_infos, source_xe, source_xa, scratch)
    }

    fn lwe_to_glwe_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.lwe_to_glwe_key_encrypt_sk_tmp_bytes(infos)
    }

    fn lwe_to_glwe_key_encrypt_sk_default<R, S1, S2, E>(
        module: &Module<BE>,
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
        module.lwe_to_glwe_key_encrypt_sk(res, sk_lwe, sk_glwe, enc_infos, source_xe, source_xa, scratch)
    }

    fn glwe_automorphism_key_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.glwe_automorphism_key_encrypt_sk_tmp_bytes(infos)
    }

    fn glwe_automorphism_key_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
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
        module.glwe_automorphism_key_encrypt_sk(res, p, sk, enc_infos, source_xe, source_xa, scratch)
    }

    fn glwe_automorphism_key_encrypt_pk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.glwe_automorphism_key_encrypt_pk_tmp_bytes(infos)
    }

    fn glwe_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GLWEInfos,
    {
        module.glwe_compressed_encrypt_sk_tmp_bytes(infos)
    }

    fn glwe_compressed_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
    {
        module.glwe_compressed_encrypt_sk(res, pt, sk, seed_xa, enc_infos, source_xe, scratch)
    }

    fn gglwe_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.gglwe_compressed_encrypt_sk_tmp_bytes(infos)
    }

    fn gglwe_compressed_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
    {
        module.gglwe_compressed_encrypt_sk(res, pt, sk, seed, enc_infos, source_xe, scratch)
    }

    fn ggsw_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGSWInfos,
    {
        module.ggsw_compressed_encrypt_sk_tmp_bytes(infos)
    }

    fn ggsw_compressed_encrypt_sk_default<'s, R, P, S, E>(
        module: &Module<BE>,
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
        for<'a> ScratchArena<'a, BE>: ScratchArenaTakeCore<'a, BE>,
    {
        module.ggsw_compressed_encrypt_sk(res, pt, sk, seed_xa, enc_infos, source_xe, scratch)
    }

    fn gglwe_to_ggsw_key_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.gglwe_to_ggsw_key_compressed_encrypt_sk_tmp_bytes(infos)
    }

    fn gglwe_to_ggsw_key_compressed_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
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
        module.gglwe_to_ggsw_key_compressed_encrypt_sk(res, sk, seed_xa, enc_infos, source_xe, scratch)
    }

    fn glwe_automorphism_key_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.glwe_automorphism_key_compressed_encrypt_sk_tmp_bytes(infos)
    }

    fn glwe_automorphism_key_compressed_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
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
        module.glwe_automorphism_key_compressed_encrypt_sk(res, p, sk, seed_xa, enc_infos, source_xe, scratch)
    }

    fn glwe_switching_key_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.glwe_switching_key_compressed_encrypt_sk_tmp_bytes(infos)
    }

    fn glwe_switching_key_compressed_encrypt_sk_default<R, S1, S2, E>(
        module: &Module<BE>,
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
        module.glwe_switching_key_compressed_encrypt_sk(res, sk_in, sk_out, seed_xa, enc_infos, source_xe, scratch)
    }

    fn glwe_tensor_key_compressed_encrypt_sk_tmp_bytes_default<A>(module: &Module<BE>, infos: &A) -> usize
    where
        A: GGLWEInfos,
    {
        module.glwe_tensor_key_compressed_encrypt_sk_tmp_bytes(infos)
    }

    fn glwe_tensor_key_compressed_encrypt_sk_default<R, S, E>(
        module: &Module<BE>,
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
        module.glwe_tensor_key_compressed_encrypt_sk(res, sk, seed_xa, enc_infos, source_xe, scratch)
    }
}
