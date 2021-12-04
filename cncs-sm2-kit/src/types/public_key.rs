use std::{error, fmt};

use num_bigint::{BigUint, ParseBigIntError};
use num_traits::Num as _;

#[derive(Clone)]
pub struct PublicKey {
    pub x: BigUint,
    pub y: BigUint,
}
impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PublicKey(x: {} y: {})",
            &self.x.to_str_radix(16).to_uppercase(),
            &self.y.to_str_radix(16).to_uppercase()
        )
    }
}

impl PublicKey {
    pub fn new(x: BigUint, y: BigUint) -> Self {
        Self { x, y }
    }

    pub fn from_bytes(x_bytes: &[u8], y_bytes: &[u8]) -> Self {
        let x = BigUint::from_bytes_be(x_bytes);
        let y = BigUint::from_bytes_be(y_bytes);
        Self { x, y }
    }

    pub fn from_hex_str(x_hex_str: &str, y_hex_str: &str) -> Result<Self, ParseBigIntError> {
        let x = BigUint::from_str_radix(x_hex_str, 16)?;
        let y = BigUint::from_str_radix(y_hex_str, 16)?;
        Ok(Self { x, y })
    }

    pub fn from_concated_hex_str(hex_str: &str) -> Result<Self, PublicKeyFromConcatedHexStrError> {
        match hex_str.len() {
            130 => {
                if !hex_str.starts_with("04") {
                    return Err(PublicKeyFromConcatedHexStrError::Invalid);
                }
                Self::from_hex_str(&hex_str[2..66], &hex_str[66..])
                    .map_err(PublicKeyFromConcatedHexStrError::ParseBigIntError)
            }
            128 => Self::from_hex_str(&hex_str[..64], &hex_str[64..])
                .map_err(PublicKeyFromConcatedHexStrError::ParseBigIntError),
            _ => Err(PublicKeyFromConcatedHexStrError::Invalid),
        }
    }

    pub fn to_concated_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.x.to_bytes_be()[..]);
        bytes.extend_from_slice(&self.y.to_bytes_be()[..]);
        bytes
    }

    pub fn to_concated_hex_str(&self) -> String {
        format!(
            "{}{}",
            &self.x.to_str_radix(16).to_uppercase(),
            &self.y.to_str_radix(16).to_uppercase()
        )
    }
}

#[derive(Debug)]
pub enum PublicKeyFromConcatedHexStrError {
    Invalid,
    ParseBigIntError(ParseBigIntError),
}
impl fmt::Display for PublicKeyFromConcatedHexStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl error::Error for PublicKeyFromConcatedHexStrError {}

//
//
//
impl TryFrom<&PublicKey> for libsm::sm2::ecc::Point {
    type Error = String;

    fn try_from(k: &PublicKey) -> Result<Self, Self::Error> {
        let curve = libsm::sm2::ecc::EccCtx::new();

        curve
            .new_point(
                &libsm::sm2::field::FieldElem::from_biguint(&k.x),
                &libsm::sm2::field::FieldElem::from_biguint(&k.y),
            )
            .map_err(|err| err.to_string())
    }
}

impl From<&PublicKey> for gmsm::g2::subject::PublicKey {
    fn from(k: &PublicKey) -> Self {
        Self {
            x: k.x.to_owned(),
            y: k.y.to_owned(),
        }
    }
}

impl From<&gmsm::g2::subject::PublicKey> for PublicKey {
    fn from(k: &gmsm::g2::subject::PublicKey) -> Self {
        Self::new(k.x.to_owned(), k.y.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{PUBLIC_KEY_X, PUBLIC_KEY_Y};

    #[test]
    fn test_from_concated_hex_str() {
        let public_key =
            PublicKey::from_concated_hex_str(format!("{}{}", PUBLIC_KEY_X, PUBLIC_KEY_Y).as_str())
                .unwrap();
        assert_eq!(
            public_key.x,
            BigUint::from_str_radix(PUBLIC_KEY_X, 16).unwrap()
        );
        assert_eq!(
            public_key.y,
            BigUint::from_str_radix(PUBLIC_KEY_Y, 16).unwrap()
        );

        //
        let public_key = PublicKey::from_concated_hex_str(
            format!("04{}{}", PUBLIC_KEY_X, PUBLIC_KEY_Y).as_str(),
        )
        .unwrap();
        assert_eq!(
            public_key.x,
            BigUint::from_str_radix(PUBLIC_KEY_X, 16).unwrap()
        );
        assert_eq!(
            public_key.y,
            BigUint::from_str_radix(PUBLIC_KEY_Y, 16).unwrap()
        );
    }

    #[test]
    fn test_get_point_for_verify() {
        let public_key = PublicKey::from_hex_str(PUBLIC_KEY_X, PUBLIC_KEY_Y).unwrap();
        for _ in 1..=3 {
            let libsm::sm2::ecc::Point { x, y, z } =
                libsm::sm2::ecc::Point::try_from(&public_key).unwrap();
            println!(
                "{} {} {}",
                x.to_biguint().to_str_radix(16).to_uppercase(),
                y.to_biguint().to_str_radix(16).to_uppercase(),
                z.to_biguint().to_str_radix(16).to_uppercase()
            );
            assert_eq!(
                x.to_biguint(),
                BigUint::from_str_radix(PUBLIC_KEY_X, 16).unwrap()
            );
            assert_eq!(
                y.to_biguint(),
                BigUint::from_str_radix(PUBLIC_KEY_Y, 16).unwrap()
            );
            assert_eq!(z.to_biguint(), BigUint::from_str_radix("1", 16).unwrap());
        }
    }
}
