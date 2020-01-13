use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io::Cursor;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct Tilt {
    pub name: String,
    pub gravity: f32,
    pub temp: f32,
}

#[derive(Debug)]
pub enum TiltError {
    Length,
    Read,
    Endian,
    Uuid,
}

fn tilt_uuids() -> HashMap<Uuid, String> {
    let mut t = HashMap::new();
    t.insert(
        "A495BB80C5B14B44B5121370F02D74DE".parse().unwrap(),
        "pink".to_owned(),
    );
    t
}

impl TryFrom<&Vec<u8>> for Tilt {
    type Error = TiltError;

    fn try_from(v: &Vec<u8>) -> Result<Self, Self::Error> {
        let arr: [u8; 25] = (&v[..]).try_into().map_err(|_| TiltError::Length)?;
        let u = Uuid::from_bytes((&v[4..20]).try_into().map_err(|_| TiltError::Length)?);
        let name = match tilt_uuids().get(&u) {
            Some(a) => a.to_owned(),
            None => return Err(TiltError::Uuid),
        };
        let mut rdr = Cursor::new(&arr[20..24]);
        let temp: f32 = rdr
            .read_u16::<BigEndian>()
            .map_err(|_| TiltError::Read)?
            .try_into()
            .map_err(|_| TiltError::Endian)?;
        let gravity: f32 = rdr
            .read_u16::<BigEndian>()
            .map_err(|_| TiltError::Read)?
            .try_into()
            .map_err(|_| TiltError::Endian)?;

        Ok(Tilt {
            name: name,
            gravity: gravity / 1000.0,
            temp: (temp - 32.0) / 1.8,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_tilt() -> Result<(), TiltError> {
        let pink_bytes: [u8; 25] = [
            76, 0, 2, 21, 164, 149, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
            222, 0, 67, 4, 4, 34,
        ];
        assert!(Tilt::try_from(&pink_bytes.to_vec()).is_ok());
        assert!(Tilt::try_from(&pink_bytes[..=23].to_vec()).is_err());
        Ok(())
    }
}
