use anyhow::Result;
use bincode::Options;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Tilt {
    pub name: String,
    pub gravity: f32,
    pub temp: f32,
}

#[derive(Debug)]
pub enum TiltErr {
    NotIbeacon,
    ButDucky,
    NotATilt,
    BadValue,
}

#[derive(Deserialize, PartialEq, Debug)]
struct RawTilt {
    _t: u8,
    _l: u8,
    name: uuid::Bytes,
    major: u16,
    minor: u16,
    _u: u8,
}

impl TryFrom<&HashMap<u16, Vec<u8>>> for RawTilt {
    type Error = TiltErr;

    fn try_from(manufacturer_data: &HashMap<u16, Vec<u8>>) -> Result<Self, Self::Error> {
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_big_endian()
            .deserialize::<RawTilt>(&ibeacon(manufacturer_data)?[..])
            .map_err(|_| TiltErr::NotATilt)
    }
}

impl TryFrom<&HashMap<u16, Vec<u8>>> for Tilt {
    type Error = TiltErr;

    fn try_from(manufacturer_data: &HashMap<u16, Vec<u8>>) -> Result<Self, Self::Error> {
        let raw = RawTilt::try_from(manufacturer_data)?;

        let temp = (raw.major as f32 - 32.0) / 1.8;
        let gravity = (raw.minor as f32) / 1000.0;

        if !(0.0..100.0).contains(&temp) {
            return Err(TiltErr::BadValue);
        }

        Ok(Tilt {
            name: tilt_name(raw.name)?,
            temp,
            gravity,
        })
    }
}

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
fn tilt_name(data: uuid::Bytes) -> Result<String, TiltErr> {
    Ok(tilt_uuids()
        .get(&Uuid::from_bytes(data))
        .ok_or(TiltErr::ButDucky)?
        .to_owned())
}

fn ibeacon(d: &HashMap<u16, Vec<u8>>) -> Result<Vec<u8>, TiltErr> {
    d.get(&76).map(|v| v.to_owned()).ok_or(TiltErr::NotIbeacon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
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
        .unwrap();

        assert_eq!(tilt.name, "Pink");
        assert_eq!(tilt.gravity, 1.028);
        assert!(f32::abs(tilt.temp - 19.4) < 0.1);
    }
}
