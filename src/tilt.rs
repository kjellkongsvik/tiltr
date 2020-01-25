use serde::Serialize;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct Tilt {
    pub name: String,
    pub gravity: f32,
    pub temp: f32,
}

#[derive(Debug)]
pub struct NotATilt;

fn tilt_uuids() -> HashMap<Uuid, String> {
    let mut t = HashMap::new();
    t.insert(
        "A495BB80C5B14B44B5121370F02D74DE".parse().unwrap(),
        "pink".to_owned(),
    );
    t
}

fn tilt_name(data: &[u8]) -> Result<String, NotATilt> {
    Ok(tilt_uuids()
        .get(&Uuid::from_bytes(data.try_into().expect("len: 16")))
        .ok_or(NotATilt)?
        .to_owned())
}

impl TryFrom<&[u8; 25]> for Tilt {
    type Error = NotATilt;

    fn try_from(data: &[u8; 25]) -> Result<Self, Self::Error> {
        let read = |data: &[u8]| u16::from_be_bytes(data.try_into().expect("len: 2")) as f32;
        Ok(Tilt {
            name: tilt_name(&data[4..20])?,
            temp: (read(&data[20..22]) - 32.0) / 1.8,
            gravity: read(&data[22..24]) / 1000.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pink_bytes() -> [u8; 25] {
        [
            76, 0, 2, 21, 164, 149, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
            222, 0, 67, 4, 4, 34,
        ]
    }

    #[test]
    fn into_tilt() {
        assert!(Tilt::try_from(&pink_bytes()).is_ok());
    }

    #[test]
    fn values() {
        let tilt = Tilt::try_from(&pink_bytes()).unwrap();
        assert_eq!(tilt.name, "pink");
        assert_eq!(tilt.gravity, 1.028);
        assert!(f32::abs(tilt.temp - 19.4) < 0.1);
    }
}
