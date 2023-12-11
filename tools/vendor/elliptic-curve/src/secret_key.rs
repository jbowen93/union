//! Secret keys for elliptic curves (i.e. private scalars).
//!
//! The [`SecretKey`] type is a wrapper around a secret scalar value which is
//! designed to prevent unintentional exposure (e.g. via `Debug` or other
//! logging). It also handles zeroing the secret value out of memory securely
//! on drop.

#[cfg(all(feature = "pkcs8", feature = "sec1"))]
mod pkcs8;

use crate::{Curve, Error, FieldBytes, Result, ScalarPrimitive};
use core::fmt::{self, Debug};
use generic_array::typenum::Unsigned;
use subtle::{Choice, ConstantTimeEq};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "arithmetic")]
use crate::{rand_core::CryptoRngCore, CurveArithmetic, NonZeroScalar, PublicKey};

#[cfg(feature = "jwk")]
use crate::jwk::{JwkEcKey, JwkParameters};

#[cfg(feature = "sec1")]
use sec1::der;

#[cfg(all(feature = "alloc", feature = "arithmetic", feature = "sec1"))]
use {
    crate::{
        sec1::{FromEncodedPoint, ToEncodedPoint},
        AffinePoint,
    },
    alloc::vec::Vec,
    sec1::der::Encode,
    zeroize::Zeroizing,
};

#[cfg(all(feature = "arithmetic", any(feature = "jwk", feature = "pem")))]
use alloc::string::String;

#[cfg(all(feature = "arithmetic", feature = "jwk"))]
use alloc::string::ToString;

#[cfg(feature = "pem")]
use pem_rfc7468 as pem;

#[cfg(feature = "sec1")]
use crate::{
    sec1::{EncodedPoint, ModulusSize, ValidatePublicKey},
    FieldBytesSize,
};

#[cfg(all(doc, feature = "pkcs8"))]
use {crate::pkcs8::DecodePrivateKey, core::str::FromStr};

/// Type label for PEM-encoded SEC1 private keys.
#[cfg(feature = "pem")]
pub(crate) const SEC1_PEM_TYPE_LABEL: &str = "EC PRIVATE KEY";

/// Elliptic curve secret keys.
///
/// This type wraps a secret scalar value, helping to prevent accidental
/// exposure and securely erasing the value from memory when dropped.
///
/// # Parsing PKCS#8 Keys
///
/// PKCS#8 is a commonly used format for encoding secret keys (especially ones
/// generated by OpenSSL).
///
/// Keys in PKCS#8 format are either binary (ASN.1 BER/DER), or PEM encoded
/// (ASCII) and begin with the following:
///
/// ```text
/// -----BEGIN PRIVATE KEY-----
/// ```
///
/// To decode an elliptic curve private key from PKCS#8, enable the `pkcs8`
/// feature of this crate (or the `pkcs8` feature of a specific RustCrypto
/// elliptic curve crate) and use the [`DecodePrivateKey`]  trait to parse it.
///
/// When the `pem` feature of this crate (or a specific RustCrypto elliptic
/// curve crate) is enabled, a [`FromStr`] impl is also available.
#[derive(Clone)]
pub struct SecretKey<C: Curve> {
    /// Scalar value
    inner: ScalarPrimitive<C>,
}

impl<C> SecretKey<C>
where
    C: Curve,
{
    /// Generate a random [`SecretKey`].
    #[cfg(feature = "arithmetic")]
    pub fn random(rng: &mut impl CryptoRngCore) -> Self
    where
        C: CurveArithmetic,
    {
        Self {
            inner: NonZeroScalar::<C>::random(rng).into(),
        }
    }

    /// Create a new secret key from a scalar value.
    pub fn new(scalar: ScalarPrimitive<C>) -> Self {
        Self { inner: scalar }
    }

    /// Borrow the inner secret [`ScalarPrimitive`] value.
    ///
    /// # ⚠️ Warning
    ///
    /// This value is key material.
    ///
    /// Please treat it with the care it deserves!
    pub fn as_scalar_primitive(&self) -> &ScalarPrimitive<C> {
        &self.inner
    }

    /// Get the secret [`NonZeroScalar`] value for this key.
    ///
    /// # ⚠️ Warning
    ///
    /// This value is key material.
    ///
    /// Please treat it with the care it deserves!
    #[cfg(feature = "arithmetic")]
    pub fn to_nonzero_scalar(&self) -> NonZeroScalar<C>
    where
        C: CurveArithmetic,
    {
        self.into()
    }

    /// Get the [`PublicKey`] which corresponds to this secret key
    #[cfg(feature = "arithmetic")]
    pub fn public_key(&self) -> PublicKey<C>
    where
        C: CurveArithmetic,
    {
        PublicKey::from_secret_scalar(&self.to_nonzero_scalar())
    }

    /// Deserialize secret key from an encoded secret scalar.
    pub fn from_bytes(bytes: &FieldBytes<C>) -> Result<Self> {
        let inner: ScalarPrimitive<C> =
            Option::from(ScalarPrimitive::from_bytes(bytes)).ok_or(Error)?;

        if inner.is_zero().into() {
            return Err(Error);
        }

        Ok(Self { inner })
    }

    /// Deserialize secret key from an encoded secret scalar passed as a
    /// byte slice.
    ///
    /// The slice is expected to be at most `C::FieldBytesSize` bytes in
    /// length but may be up to 4-bytes shorter than that, which is handled by
    /// zero-padding the value.
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() > C::FieldBytesSize::USIZE {
            return Err(Error);
        }

        /// Maximum number of "missing" bytes to interpret as zeroes.
        const MAX_LEADING_ZEROES: usize = 4;

        let offset = C::FieldBytesSize::USIZE.saturating_sub(slice.len());

        if offset == 0 {
            Self::from_bytes(FieldBytes::<C>::from_slice(slice))
        } else if offset <= MAX_LEADING_ZEROES {
            let mut bytes = FieldBytes::<C>::default();
            bytes[offset..].copy_from_slice(slice);

            let ret = Self::from_bytes(&bytes);
            bytes.zeroize();
            ret
        } else {
            Err(Error)
        }
    }

    /// Serialize raw secret scalar as a big endian integer.
    pub fn to_bytes(&self) -> FieldBytes<C> {
        self.inner.to_bytes()
    }

    /// Deserialize secret key encoded in the SEC1 ASN.1 DER `ECPrivateKey` format.
    #[cfg(all(feature = "sec1"))]
    pub fn from_sec1_der(der_bytes: &[u8]) -> Result<Self>
    where
        C: Curve + ValidatePublicKey,
        FieldBytesSize<C>: ModulusSize,
    {
        sec1::EcPrivateKey::try_from(der_bytes)?
            .try_into()
            .map_err(|_| Error)
    }

    /// Serialize secret key in the SEC1 ASN.1 DER `ECPrivateKey` format.
    #[cfg(all(feature = "alloc", feature = "arithmetic", feature = "sec1"))]
    pub fn to_sec1_der(&self) -> der::Result<Zeroizing<Vec<u8>>>
    where
        C: CurveArithmetic,
        AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
        FieldBytesSize<C>: ModulusSize,
    {
        let private_key_bytes = Zeroizing::new(self.to_bytes());
        let public_key_bytes = self.public_key().to_encoded_point(false);

        let ec_private_key = Zeroizing::new(
            sec1::EcPrivateKey {
                private_key: &private_key_bytes,
                parameters: None,
                public_key: Some(public_key_bytes.as_bytes()),
            }
            .to_der()?,
        );

        Ok(ec_private_key)
    }

    /// Parse [`SecretKey`] from PEM-encoded SEC1 `ECPrivateKey` format.
    ///
    /// PEM-encoded SEC1 keys can be identified by the leading delimiter:
    ///
    /// ```text
    /// -----BEGIN EC PRIVATE KEY-----
    /// ```
    #[cfg(feature = "pem")]
    pub fn from_sec1_pem(s: &str) -> Result<Self>
    where
        C: Curve + ValidatePublicKey,
        FieldBytesSize<C>: ModulusSize,
    {
        let (label, der_bytes) = pem::decode_vec(s.as_bytes()).map_err(|_| Error)?;

        if label != SEC1_PEM_TYPE_LABEL {
            return Err(Error);
        }

        Self::from_sec1_der(&der_bytes).map_err(|_| Error)
    }

    /// Serialize private key as self-zeroizing PEM-encoded SEC1 `ECPrivateKey`
    /// with the given [`pem::LineEnding`].
    ///
    /// Pass `Default::default()` to use the OS's native line endings.
    #[cfg(feature = "pem")]
    pub fn to_sec1_pem(&self, line_ending: pem::LineEnding) -> Result<Zeroizing<String>>
    where
        C: CurveArithmetic,
        AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
        FieldBytesSize<C>: ModulusSize,
    {
        self.to_sec1_der()
            .ok()
            .and_then(|der| pem::encode_string(SEC1_PEM_TYPE_LABEL, line_ending, &der).ok())
            .map(Zeroizing::new)
            .ok_or(Error)
    }

    /// Parse a [`JwkEcKey`] JSON Web Key (JWK) into a [`SecretKey`].
    #[cfg(feature = "jwk")]
    pub fn from_jwk(jwk: &JwkEcKey) -> Result<Self>
    where
        C: JwkParameters + ValidatePublicKey,
        FieldBytesSize<C>: ModulusSize,
    {
        Self::try_from(jwk)
    }

    /// Parse a string containing a JSON Web Key (JWK) into a [`SecretKey`].
    #[cfg(feature = "jwk")]
    pub fn from_jwk_str(jwk: &str) -> Result<Self>
    where
        C: JwkParameters + ValidatePublicKey,
        FieldBytesSize<C>: ModulusSize,
    {
        jwk.parse::<JwkEcKey>().and_then(|jwk| Self::from_jwk(&jwk))
    }

    /// Serialize this secret key as [`JwkEcKey`] JSON Web Key (JWK).
    #[cfg(all(feature = "arithmetic", feature = "jwk"))]
    pub fn to_jwk(&self) -> JwkEcKey
    where
        C: CurveArithmetic + JwkParameters,
        AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
        FieldBytesSize<C>: ModulusSize,
    {
        self.into()
    }

    /// Serialize this secret key as JSON Web Key (JWK) string.
    #[cfg(all(feature = "arithmetic", feature = "jwk"))]
    pub fn to_jwk_string(&self) -> Zeroizing<String>
    where
        C: CurveArithmetic + JwkParameters,
        AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
        FieldBytesSize<C>: ModulusSize,
    {
        Zeroizing::new(self.to_jwk().to_string())
    }
}

impl<C> ConstantTimeEq for SecretKey<C>
where
    C: Curve,
{
    fn ct_eq(&self, other: &Self) -> Choice {
        self.inner.ct_eq(&other.inner)
    }
}

impl<C> Debug for SecretKey<C>
where
    C: Curve,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(core::any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}

impl<C> ZeroizeOnDrop for SecretKey<C> where C: Curve {}

impl<C> Drop for SecretKey<C>
where
    C: Curve,
{
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl<C: Curve> Eq for SecretKey<C> {}

impl<C> PartialEq for SecretKey<C>
where
    C: Curve,
{
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

#[cfg(all(feature = "sec1"))]
impl<C> TryFrom<sec1::EcPrivateKey<'_>> for SecretKey<C>
where
    C: Curve + ValidatePublicKey,
    FieldBytesSize<C>: ModulusSize,
{
    type Error = der::Error;

    fn try_from(sec1_private_key: sec1::EcPrivateKey<'_>) -> der::Result<Self> {
        let secret_key = Self::from_slice(sec1_private_key.private_key)
            .map_err(|_| der::Tag::Sequence.value_error())?;

        // TODO(tarcieri): validate `sec1_private_key.params`?
        if let Some(pk_bytes) = sec1_private_key.public_key {
            let pk = EncodedPoint::<C>::from_bytes(pk_bytes)
                .map_err(|_| der::Tag::BitString.value_error())?;

            if C::validate_public_key(&secret_key, &pk).is_err() {
                return Err(der::Tag::BitString.value_error());
            }
        }

        Ok(secret_key)
    }
}

#[cfg(feature = "arithmetic")]
impl<C> From<NonZeroScalar<C>> for SecretKey<C>
where
    C: CurveArithmetic,
{
    fn from(scalar: NonZeroScalar<C>) -> SecretKey<C> {
        SecretKey::from(&scalar)
    }
}

#[cfg(feature = "arithmetic")]
impl<C> From<&NonZeroScalar<C>> for SecretKey<C>
where
    C: CurveArithmetic,
{
    fn from(scalar: &NonZeroScalar<C>) -> SecretKey<C> {
        SecretKey {
            inner: scalar.into(),
        }
    }
}
