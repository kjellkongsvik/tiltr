use std::collections::HashMap;
use std::convert::TryFrom;

type Bytes = [u8; 16];
type IBeacon = (u8, u8, Bytes, u16, u16, u8);

#[derive(Debug, PartialEq)]
pub struct Tilt {
    pub name: String,
    pub gravity: f32,
    pub temp: f32,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum TiltError {
    #[error("Not IBeacon")]
    NotIBeacon,
    #[error("Not tilt")]
    NotTilt,
    #[error("Unexpected temp value")]
    UnexpectedTempValue,
    #[error("Unexpected gravity value")]
    UnexpectedGravityValue,
}

impl TryFrom<&Vec<u8>> for Tilt {
    type Error = TiltError;

    fn try_from(data: &Vec<u8>) -> Result<Self, TiltError> {
        if data.len() != 23 {
            return Err(TiltError::NotIBeacon);
        }
        let config = bincode::config::standard()
            .with_fixed_int_encoding()
            .with_big_endian();
        let (ibeacon, _): (IBeacon, _) = bincode::decode_from_slice(data, config)
            .map_err(|_| TiltError::NotIBeacon)?;
        let name = my_tilts().get(&ibeacon.2).ok_or(TiltError::NotTilt)?.into();
        let temp = (f32::from(ibeacon.3) - 32.0) / 1.8;
        if !(0.0..100.0).contains(&temp) {
            return Err(TiltError::UnexpectedTempValue);
        }

        let gravity = f32::from(ibeacon.4) / 1000.0;
        if !(0.9..1.1).contains(&gravity) {
            return Err(TiltError::UnexpectedGravityValue);
        }

        Ok(Tilt {
            name,
            gravity,
            temp,
        })
    }
}

fn my_tilts() -> HashMap<Bytes, String> {
    let mut tilts = HashMap::new();

    let mut add_tilt = |id: Bytes, name: &str| tilts.insert(id, name.into());

    add_tilt(pink(), "Pink");

    tilts
}

fn pink() -> Bytes {
    u128::from_str_radix("a495bb80c5b14b44b5121370f02d74de", 16)
        .unwrap()
        .to_be_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unknown() -> Bytes {
        u128::from_str_radix("00000000000000000000000000000000", 16)
            .unwrap()
            .to_be_bytes()
    }

    #[test]
    fn bad_temp_100() {
        let mut bytes = vec![0, 0];
        bytes.extend(pink());
        bytes.extend([0, 212, 4, 4, 34]);
        let tilt = Tilt::try_from(&bytes);
        assert_eq!(tilt, Err(TiltError::UnexpectedTempValue));
    }

    #[test]
    fn bad_temp_0() {
        let mut bytes = vec![0, 0];
        bytes.extend(pink());
        bytes.extend([0, 31, 4, 4, 34]);
        let tilt = Tilt::try_from(&bytes);
        assert_eq!(tilt, Err(TiltError::UnexpectedTempValue));
    }

    #[test]
    fn bad_g_08() {
        let mut bytes = vec![0, 0];
        bytes.extend(pink());
        bytes.extend([0, 67, 3, 32, 34]);
        let tilt = Tilt::try_from(&bytes);
        assert_eq!(tilt, Err(TiltError::UnexpectedGravityValue));
    }

    #[test]
    fn bad_g_12() {
        let mut bytes = vec![0, 0];
        bytes.extend(pink());
        bytes.extend([0, 67, 4, 176, 34]);
        let tilt = Tilt::try_from(&bytes);
        assert_eq!(tilt, Err(TiltError::UnexpectedGravityValue));
    }

    #[test]
    fn not_ibeacon() {
        let bytes = vec![0];
        let tilt = Tilt::try_from(&bytes);

        assert_eq!(tilt, Err(TiltError::NotIBeacon));
    }

    #[test]
    fn not_tilt() {
        let mut bytes = vec![0, 0];
        bytes.extend(unknown());
        bytes.extend([0, 0, 0, 0, 0]);
        let tilt = Tilt::try_from(&bytes);

        assert_eq!(tilt, Err(TiltError::NotTilt));
    }

    #[test]
    fn happy() {
        let mut bytes = vec![0, 0];
        bytes.extend(pink());
        bytes.extend([0, 67, 4, 4, 0]);
        let tilt = Tilt::try_from(&bytes).expect("Valid tilt");

        assert_eq!(tilt.name, "Pink");
        assert_eq!(tilt.gravity, 1.028);
        assert!(f32::abs(tilt.temp - 19.4) < 0.1);
    }
}
