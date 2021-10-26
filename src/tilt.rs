use bincode::Options;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use uuid::Uuid;

#[derive(Serialize, Debug)]
pub struct Tilt {
    pub name: String,
    pub gravity: f32,
    pub temp: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not an ibeacon device")]
    NotIbeacon,
    #[error("Not a tilt")]
    NotATilt,
    #[error("Unexpected temp value")]
    UnexpectedTempValue,
    #[error("Unexpected gravity value")]
    UnexpectedGravityValue,
}

#[derive(Deserialize)]
struct RawTilt {
    _t: u8,
    _l: u8,
    name: uuid::Bytes,
    major: u16,
    minor: u16,
    _u: u8,
}

impl TryFrom<&HashMap<u16, Vec<u8>>> for RawTilt {
    type Error = Error;

    fn try_from(manufacturer_data: &HashMap<u16, Vec<u8>>) -> Result<Self, Error> {
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_big_endian()
            .deserialize::<RawTilt>(&ibeacon(manufacturer_data)?[..])
            .map_err(|_| Error::NotATilt)
    }
}

impl TryFrom<&HashMap<u16, Vec<u8>>> for Tilt {
    type Error = Error;

    fn try_from(manufacturer_data: &HashMap<u16, Vec<u8>>) -> Result<Self, Error> {
        let raw = RawTilt::try_from(manufacturer_data)?;

        let name = known_tilt_name(raw.name)?;

        let temp = (f32::from(raw.major) - 32.0) / 1.8;
        if !(0.0..100.0).contains(&temp) {
            return Err(Error::UnexpectedTempValue);
        }

        let gravity = f32::from(raw.minor) / 1000.0;
        if !(0.9..1.1).contains(&gravity) {
            return Err(Error::UnexpectedGravityValue);
        }

        Ok(Tilt { name, gravity, temp })
    }
}

// TODO simple but ugly
fn tilt_uuids() -> HashMap<Uuid, String> {
    "a495bb10c5b14b44b5121370f02d74de,Red
a495bb20c5b14b44b5121370f02d74de,Green
a495bb30c5b14b44b5121370f02d74de,Black
a495bb40c5b14b44b5121370f02d74de,Purple
a495bb50c5b14b44b5121370f02d74de,Orange
a495bb60c5b14b44b5121370f02d74de,Blue
a495bb70c5b14b44b5121370f02d74de,Yellow
a495bb80c5b14b44b5121370f02d74de,Pink"
        .lines()
        .map(|l| l.split(','))
        .fold(HashMap::new(), |mut hm, mut l| {
            hm.entry(l.next().unwrap().parse().unwrap())
                .or_insert_with(|| l.next().unwrap().to_string());
            hm
        })
}

fn known_tilt_name(data: uuid::Bytes) -> Result<String, Error> {
    Ok(tilt_uuids()
        .get(&Uuid::from_bytes(data))
        .ok_or(Error::NotATilt)?
        .clone())
}

fn ibeacon(d: &HashMap<u16, Vec<u8>>) -> Result<Vec<u8>, Error> {
    match d.get(&76).cloned().ok_or(Error::NotIbeacon) {
        Ok(v) if v.len() != 23 => Err(Error::NotIbeacon),
        Ok(v) if v[1] != 21 => Err(Error::NotIbeacon),
        Ok(v) => Ok(v),
        _ => Err(Error::NotIbeacon),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_err {
        ($expression:expr, $($pattern:tt)+) => {
            match $expression {
                $($pattern)+ => (),
                ref e => panic!("expected `{}` but got `{:?}`", stringify!($($pattern)+), e),
            }
        }
    }

    #[test]
    fn not_ibeacon() {
        let tilt = Tilt::try_from(&[(77, vec![])].iter().cloned().collect());
        assert_err!(tilt, Err(Error::NotIbeacon));

        let tilt = Tilt::try_from(&[(76, vec![])].iter().cloned().collect());
        assert_err!(tilt, Err(Error::NotIbeacon));

        let tilt = Tilt::try_from(&[(76, vec![0; 23])].iter().cloned().collect());
        assert_err!(tilt, Err(Error::NotIbeacon));
    }

    #[test]
    fn unknown_uuid() {
        let tilt = Tilt::try_from(&[(76, vec![21; 23])].iter().cloned().collect());
        assert_err!(tilt, Err(Error::NotATilt));
    }

    #[test]
    fn bad_temp_100() {
        let tilt = Tilt::try_from(
            &[(
                76,
                vec![
                    2, 21, 164, 149, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
                    222, 0, 212, 4, 4, 34,
                ],
            )]
            .iter()
            .cloned()
            .collect(),
        );
        assert_err!(tilt, Err(Error::UnexpectedTempValue));
    }

    #[test]
    fn bad_temp_0() {
        let tilt = Tilt::try_from(
            &[(
                76,
                vec![
                    2, 21, 164, 149, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
                    222, 0, 31, 4, 4, 34,
                ],
            )]
            .iter()
            .cloned()
            .collect(),
        );
        assert_err!(tilt, Err(Error::UnexpectedTempValue));
    }

    #[test]
    fn bad_g_08() {
        let tilt = Tilt::try_from(
            &[(
                76,
                vec![
                    2, 21, 164, 149, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
                    222, 0, 67, 3, 32, 34,
                ],
            )]
            .iter()
            .cloned()
            .collect(),
        );
        assert_err!(tilt, Err(Error::UnexpectedGravityValue));
    }

    #[test]
    fn bad_g_12() {
        let tilt = Tilt::try_from(
            &[(
                76,
                vec![
                    2, 21, 164, 149, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
                    222, 0, 67, 4, 176, 34,
                ],
            )]
            .iter()
            .cloned()
            .collect(),
        );
        assert_err!(tilt, Err(Error::UnexpectedGravityValue));
    }

    #[test]
    fn happy() {
        let tilt = Tilt::try_from(
            &[(
                76,
                vec![
                    2, 21, 164, 149, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
                    222, 0, 67, 4, 4, 34,
                ],
            )]
            .iter()
            .cloned()
            .collect(),
        )
        .expect("Valid sample");

        assert_eq!(tilt.name, "Pink");
        assert_eq!(tilt.gravity, 1.028);
        assert!(f32::abs(tilt.temp - 19.4) < 0.1);
    }
}
