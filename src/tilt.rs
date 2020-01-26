use serde::Serialize;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Tilt {
    pub name: String,
    pub gravity: f32,
    pub temp: f32,
}

#[derive(Debug)]
pub struct NotATilt;

fn tilt_list() -> String {
    "a495bb10c5b14b44b5121370f02d74de,Red
a495bb20c5b14b44b5121370f02d74de,Green
a495bb30c5b14b44b5121370f02d74de,Black
a495bb40c5b14b44b5121370f02d74de,Purple
a495bb50c5b14b44b5121370f02d74de,Orange
a495bb60c5b14b44b5121370f02d74de,Blue
a495bb70c5b14b44b5121370f02d74de,Yellow
a495bb80c5b14b44b5121370f02d74de,Pink"
        .to_string()
}

fn tilt_uuids(s: &str) -> HashMap<Uuid, String> {
    s.lines()
        .map(|l| l.split(','))
        .fold(HashMap::new(), |mut hm, mut l| {
            hm.entry(l.next().unwrap().parse().unwrap())
                .or_insert_with(|| l.next().unwrap().to_string());
            hm
        })
}

fn tilt_name(data: &[u8]) -> Result<String, NotATilt> {
    Ok(tilt_uuids(&tilt_list())
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
        assert_eq!(tilt.name, "Pink");
        assert_eq!(tilt.gravity, 1.028);
        assert!(f32::abs(tilt.temp - 19.4) < 0.1);
    }
}
