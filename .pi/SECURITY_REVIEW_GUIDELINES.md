## Security categories to examine

Poulpy is a fully homomorphic encryption (FHE) library. Focus on FHE specific security bugs: voilation of learning with errors assumption, bias in sampling of secrets/noise, encryption, decryption, un-intentional leakage of secret key or noise (IND-CPA^D type bugs).

*Secrets/Noise are sampled at random*
- All secret (variants of secret key) and noise vectors are sampled from said distribution with no-bias and secure randomness.
- Same RNG state is not used to sample two distinct values/vectors.
