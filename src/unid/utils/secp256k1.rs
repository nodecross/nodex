//! Pure Rust implementation of the secp256k1 curve and fast ECDSA
//! signatures. The secp256k1 curve is used extensively in Bitcoin and
//! Ethereum-alike cryptocurrencies.

#![deny(
    unused_import_braces,
    unused_imports,
    unused_comparisons,
    unused_must_use,
    unused_variables,
    non_shorthand_field_patterns,
    unreachable_code,
    unused_parens
)]


// pub use libsecp256k1_core::*;

use alloc::boxed::Box;
use arrayref::{array_mut_ref, array_ref};
use core::convert::TryFrom;
use hmac_drbg::HmacDRBG;
use sha2::Sha256;
use typenum::U32;
use crate::MUTEX_HANDLERS;
use alloc::format;


use crate::unid::utils::libsecp256k1_core::{
  util,
  error::Error,
  field::Field,
  group::Affine,
  scalar::Scalar,
  ecmult::{ECMultContext, ECMultGenContext},
};
// pub static mut ECMULT_CONTEXT: Lazy<Mutex<Box<ECMultContext>>> = Lazy::new(|| {
//   Mutex::new(ECMultContext::new_boxed())
// });

// pub static mut ECMULT_GEN_CONTEXT: Lazy<Mutex<Box<ECMultGenContext>>> = Lazy::new(|| {
//   Mutex::new(ECMultGenContext::new_boxed())
// });


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Public key on a secp256k1 curve.
pub struct PublicKey(pub Affine);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Secret key (256-bit) on a secp256k1 curve.
pub struct SecretKey(Scalar);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// An ECDSA signature.
pub struct Signature {
    pub r: Scalar,
    pub s: Scalar,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Tag used for public key recovery from signatures.
pub struct RecoveryId(u8);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Hashed message input to an ECDSA signature.
pub struct Message(pub Scalar);

/// Format for public key parsing.
pub enum PublicKeyFormat {
    /// Compressed public key, 33 bytes.
    Compressed,
    /// Full length public key, 65 bytes.
    Full,
    /// Raw public key, 64 bytes.
    Raw,
}

impl PublicKey {
    pub fn parse_slice(p: &[u8], format: Option<PublicKeyFormat>) -> Result<PublicKey, Error> {
        let format = match (p.len(), format) {
            (util::FULL_PUBLIC_KEY_SIZE, None)
            | (util::FULL_PUBLIC_KEY_SIZE, Some(PublicKeyFormat::Full)) => PublicKeyFormat::Full,
            (util::COMPRESSED_PUBLIC_KEY_SIZE, None)
            | (util::COMPRESSED_PUBLIC_KEY_SIZE, Some(PublicKeyFormat::Compressed)) => {
                PublicKeyFormat::Compressed
            }
            (util::RAW_PUBLIC_KEY_SIZE, None)
            | (util::RAW_PUBLIC_KEY_SIZE, Some(PublicKeyFormat::Raw)) => PublicKeyFormat::Raw,
            _ => return Err(Error::InvalidInputLength),
        };

        match format {
            PublicKeyFormat::Full => {
                let mut a = [0; util::FULL_PUBLIC_KEY_SIZE];
                a.copy_from_slice(p);
                Self::parse(&a)
            }
            PublicKeyFormat::Raw => {
                use util::TAG_PUBKEY_FULL;

                let mut a = [0; util::FULL_PUBLIC_KEY_SIZE];
                a[0] = TAG_PUBKEY_FULL;
                a[1..].copy_from_slice(p);
                Self::parse(&a)
            }
            PublicKeyFormat::Compressed => {
                let mut a = [0; util::COMPRESSED_PUBLIC_KEY_SIZE];
                a.copy_from_slice(p);
                Self::parse_compressed(&a)
            }
        }
    }

    pub fn parse(p: &[u8; util::FULL_PUBLIC_KEY_SIZE]) -> Result<PublicKey, Error> {
        use util::{TAG_PUBKEY_FULL, TAG_PUBKEY_HYBRID_EVEN, TAG_PUBKEY_HYBRID_ODD};

        if !(p[0] == TAG_PUBKEY_FULL
            || p[0] == TAG_PUBKEY_HYBRID_EVEN
            || p[0] == TAG_PUBKEY_HYBRID_ODD)
        {
            return Err(Error::InvalidPublicKey);
        }
        let mut x = Field::default();
        let mut y = Field::default();
        if !x.set_b32(array_ref!(p, 1, 32)) {
            return Err(Error::InvalidPublicKey);
        }
        if !y.set_b32(array_ref!(p, 33, 32)) {
            return Err(Error::InvalidPublicKey);
        }
        let mut elem = Affine::default();
        elem.set_xy(&x, &y);
        if (p[0] == TAG_PUBKEY_HYBRID_EVEN || p[0] == TAG_PUBKEY_HYBRID_ODD)
            && (y.is_odd() != (p[0] == TAG_PUBKEY_HYBRID_ODD))
        {
            return Err(Error::InvalidPublicKey);
        }
        if elem.is_infinity() {
            return Err(Error::InvalidPublicKey);
        }
        if elem.is_valid_var() {
            Ok(PublicKey(elem))
        } else {
            Err(Error::InvalidPublicKey)
        }
    }

    pub fn parse_compressed(
        p: &[u8; util::COMPRESSED_PUBLIC_KEY_SIZE],
    ) -> Result<PublicKey, Error> {
        use util::{TAG_PUBKEY_EVEN, TAG_PUBKEY_ODD};

        if !(p[0] == TAG_PUBKEY_EVEN || p[0] == TAG_PUBKEY_ODD) {
            return Err(Error::InvalidPublicKey);
        }
        let mut x = Field::default();
        if !x.set_b32(array_ref!(p, 1, 32)) {
            return Err(Error::InvalidPublicKey);
        }
        let mut elem = Affine::default();
        elem.set_xo_var(&x, p[0] == TAG_PUBKEY_ODD);
        if elem.is_infinity() {
            return Err(Error::InvalidPublicKey);
        }
        if elem.is_valid_var() {
            Ok(PublicKey(elem))
        } else {
            Err(Error::InvalidPublicKey)
        }
    }

    pub fn serialize(&self) -> [u8; util::FULL_PUBLIC_KEY_SIZE] {
        use util::TAG_PUBKEY_FULL;

        debug_assert!(!self.0.is_infinity());

        let mut ret = [0u8; 65];
        let mut elem = self.0;

        elem.x.normalize_var();
        elem.y.normalize_var();
        elem.x.fill_b32(array_mut_ref!(ret, 1, 32));
        elem.y.fill_b32(array_mut_ref!(ret, 33, 32));
        ret[0] = TAG_PUBKEY_FULL;

        ret
    }

    pub fn serialize_compressed(&self) -> [u8; util::COMPRESSED_PUBLIC_KEY_SIZE] {
        use util::{TAG_PUBKEY_EVEN, TAG_PUBKEY_ODD};

        debug_assert!(!self.0.is_infinity());

        let mut ret = [0u8; 33];
        let mut elem = self.0;

        elem.x.normalize_var();
        elem.y.normalize_var();
        elem.x.fill_b32(array_mut_ref!(ret, 1, 32));
        ret[0] = if elem.y.is_odd() {
            TAG_PUBKEY_ODD
        } else {
            TAG_PUBKEY_EVEN
        };

        ret
    }

}

impl Into<Affine> for PublicKey {
    fn into(self) -> Affine {
        self.0
    }
}

impl TryFrom<Affine> for PublicKey {
    type Error = Error;

    fn try_from(value: Affine) -> Result<Self, Self::Error> {
        if value.is_infinity() || !value.is_valid_var() {
            Err(Error::InvalidAffine)
        } else {
            Ok(PublicKey(value))
        }
    }
}




impl SecretKey {
    pub fn parse(p: &[u8; util::SECRET_KEY_SIZE]) -> Result<SecretKey, Error> {
        let mut elem = Scalar::default();
        if !bool::from(elem.set_b32(p)) {
            Self::try_from(elem)
        } else {
            Err(Error::InvalidSecretKey)
        }
    }

    pub fn parse_slice(p: &[u8]) -> Result<SecretKey, Error> {
        if p.len() != util::SECRET_KEY_SIZE {
            return Err(Error::InvalidInputLength);
        }

        let mut a = [0; 32];
        a.copy_from_slice(p);
        Self::parse(&a)
    }

    pub fn serialize(&self) -> [u8; util::SECRET_KEY_SIZE] {
        self.0.b32()
    }

    pub fn inv(&self) -> Self {
        SecretKey(self.0.inv())
    }
}

impl Default for SecretKey {
    fn default() -> SecretKey {
        let mut elem = Scalar::default();
        let overflowed = bool::from(elem.set_b32(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ]));
        debug_assert!(!overflowed);
        debug_assert!(!elem.is_zero());
        SecretKey(elem)
    }
}

impl Into<Scalar> for SecretKey {
    fn into(self) -> Scalar {
        self.0
    }
}

impl TryFrom<Scalar> for SecretKey {
    type Error = Error;

    fn try_from(scalar: Scalar) -> Result<Self, Error> {
        if scalar.is_zero() {
            Err(Error::InvalidSecretKey)
        } else {
            Ok(Self(scalar))
        }
    }
}

impl core::fmt::LowerHex for SecretKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let scalar = self.0;

        write!(f, "{:x}", scalar)
    }
}

impl Signature {
    /// Parse an possibly overflowing signature.
    ///
    /// A SECP256K1 signature is usually required to be within 0 and curve
    /// order. This function, however, allows signatures larger than curve order
    /// by taking the signature and minus curve order.
    ///
    /// Note that while this function is technically safe, it is non-standard,
    /// meaning you will have compatibility issues if you also use other
    /// SECP256K1 libraries. It's not recommended to use this function. Please
    /// use `parse_standard` instead.
    pub fn parse_overflowing(p: &[u8; util::SIGNATURE_SIZE]) -> Signature {
        let mut r = Scalar::default();
        let mut s = Scalar::default();

        // Okay for signature to overflow
        let _ = r.set_b32(array_ref!(p, 0, 32));
        let _ = s.set_b32(array_ref!(p, 32, 32));

        Signature { r, s }
    }

    /// Parse a standard SECP256K1 signature. The signature is required to be
    /// within 0 and curve order. Returns error if it overflows.
    pub fn parse_standard(p: &[u8; util::SIGNATURE_SIZE]) -> Result<Signature, Error> {
        let mut r = Scalar::default();
        let mut s = Scalar::default();

        // It's okay for the signature to overflow here, it's checked below.
        let overflowed_r = r.set_b32(array_ref!(p, 0, 32));
        let overflowed_s = s.set_b32(array_ref!(p, 32, 32));

        if bool::from(overflowed_r | overflowed_s) {
            return Err(Error::InvalidSignature);
        }

        Ok(Signature { r, s })
    }

    /// Parse a possibly overflowing signature slice. See also
    /// `parse_overflowing`.
    ///
    /// It's not recommended to use this function. Please use
    /// `parse_standard_slice` instead.
    pub fn parse_overflowing_slice(p: &[u8]) -> Result<Signature, Error> {
        if p.len() != util::SIGNATURE_SIZE {
            return Err(Error::InvalidInputLength);
        }

        let mut a = [0; util::SIGNATURE_SIZE];
        a.copy_from_slice(p);
        Ok(Self::parse_overflowing(&a))
    }

    /// Parse a standard signature slice. See also `parse_standard`.
    pub fn parse_standard_slice(p: &[u8]) -> Result<Signature, Error> {
        if p.len() != util::SIGNATURE_SIZE {
            return Err(Error::InvalidInputLength);
        }

        let mut a = [0; util::SIGNATURE_SIZE];
        a.copy_from_slice(p);
        Ok(Self::parse_standard(&a)?)
    }


    /// Normalizes a signature to a "low S" form. In ECDSA, signatures are
    /// of the form (r, s) where r and s are numbers lying in some finite
    /// field. The verification equation will pass for (r, s) iff it passes
    /// for (r, -s), so it is possible to ``modify'' signatures in transit
    /// by flipping the sign of s. This does not constitute a forgery since
    /// the signed message still cannot be changed, but for some applications,
    /// changing even the signature itself can be a problem. Such applications
    /// require a "strong signature". It is believed that ECDSA is a strong
    /// signature except for this ambiguity in the sign of s, so to accommodate
    /// these applications libsecp256k1 will only accept signatures for which
    /// s is in the lower half of the field range. This eliminates the
    /// ambiguity.
    ///
    /// However, for some systems, signatures with high s-values are considered
    /// valid. (For example, parsing the historic Bitcoin blockchain requires
    /// this.) For these applications we provide this normalization function,
    /// which ensures that the s value lies in the lower half of its range.
    pub fn normalize_s(&mut self) {
        if self.s.is_high() {
            self.s = -self.s;
        }
    }

    /// Serialize a signature to a standard byte representation. This is the
    /// reverse of `parse_standard`.
    pub fn serialize(&self) -> [u8; util::SIGNATURE_SIZE] {
        let mut ret = [0u8; 64];
        self.r.fill_b32(array_mut_ref!(ret, 0, 32));
        self.s.fill_b32(array_mut_ref!(ret, 32, 32));
        ret
    }

}

impl Message {
    pub fn parse(p: &[u8; util::MESSAGE_SIZE]) -> Message {
        let mut m = Scalar::default();

        // Okay for message to overflow.
        let _ = m.set_b32(p);

        Message(m)
    }

    pub fn parse_slice(p: &[u8]) -> Result<Message, Error> {
        if p.len() != util::MESSAGE_SIZE {
            return Err(Error::InvalidInputLength);
        }

        let mut a = [0; util::MESSAGE_SIZE];
        a.copy_from_slice(p);
        Ok(Self::parse(&a))
    }

    pub fn serialize(&self) -> [u8; util::MESSAGE_SIZE] {
        self.0.b32()
    }
}

impl RecoveryId {
    /// Parse recovery ID starting with 0.
    pub fn parse(p: u8) -> Result<RecoveryId, Error> {
        if p < 4 {
            Ok(RecoveryId(p))
        } else {
            Err(Error::InvalidRecoveryId)
        }
    }

    /// Parse recovery ID as Ethereum RPC format, starting with 27.
    pub fn parse_rpc(p: u8) -> Result<RecoveryId, Error> {
        if p >= 27 && p < 27 + 4 {
            RecoveryId::parse(p - 27)
        } else {
            Err(Error::InvalidRecoveryId)
        }
    }

    pub fn serialize(&self) -> u8 {
        self.0
    }
}

impl Into<u8> for RecoveryId {
    fn into(self) -> u8 {
        self.0
    }
}

impl Into<i32> for RecoveryId {
    fn into(self) -> i32 {
        self.0 as i32
    }
}


pub fn verify_with_context(
    message: &Message,
    signature: &Signature,
    pubkey: &PublicKey,
    context: &ECMultContext,
) -> bool {
    context.verify_raw(&signature.r, &signature.s, &pubkey.0, &message.0)
}


pub fn verify(message: &Message, signature: &Signature, pubkey: &PublicKey) -> bool {
  let ECMULT_CONTEXT: Box<ECMultContext> = ECMultContext::new_boxed();
  // let ECMULT_CONTEXT: ECMultContext =
  // unsafe { ECMultContext::new_from_raw(include!("out/const.rs")) };
  verify_with_context(message, signature, pubkey, &ECMULT_CONTEXT)
}



pub fn sign_with_context(
    message: &Message,
    seckey: &SecretKey,
    context: &ECMultGenContext,
) -> (Signature, RecoveryId) {
    let seckey_b32 = seckey.0.b32();
    let message_b32 = message.0.b32();
    unsafe {
      let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());
  
      logger.debug(format!("message_b32 = {:?}", message_b32 ));
    }

    let mut drbg = HmacDRBG::<Sha256>::new(&seckey_b32, &message_b32, &[]);
    unsafe {
      let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());
  
      logger.debug(format!("hmac drbg"));
    }
    let mut nonce = Scalar::default();
    let mut overflow;

    let result;

    loop {
        let generated = drbg.generate::<U32>(None);
        overflow = bool::from(nonce.set_b32(array_ref!(generated, 0, 32)));

        if !overflow && !nonce.is_zero() {
          unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());
        
            logger.debug(format!("not overflow"));
          }
            if let Ok(val) = context.sign_raw(&seckey.0, &message.0, &nonce) {
                result = val;
                unsafe {
                  let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());
              
                  logger.debug(format!("result now = {:?}",result));
                }
                break;
            }
        }
    }

    #[allow(unused_assignments)]
    {
        nonce = Scalar::default();
    }
    let (sigr, sigs, recid) = result;

    (Signature { r: sigr, s: sigs }, RecoveryId(recid))
}


pub fn sign(message: &Message, seckey: &SecretKey) -> (Signature, RecoveryId) {  


  let ECMULT_GEN_CONTEXT: Box<ECMultGenContext> = ECMultGenContext::new_boxed();
  // let ECMULT_GEN_CONTEXT: ECMultGenContext =
  // unsafe { ECMultGenContext::new_from_raw(include_str!("out/const_gen.rs")) };
  
  sign_with_context(message, seckey, &ECMULT_GEN_CONTEXT)
}
