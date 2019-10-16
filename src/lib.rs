use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Cursor;
use uuid::Uuid;

#[derive(Debug)]
pub struct Tilt {
    pub uuid: Uuid,
    pub t: f32,
    pub sg: f32,
}

fn from(item: &Vec<u8>) -> Option<Tilt> {
    let arr: [u8; 25] = (&item[..]).try_into().ok()?;
    let u = Uuid::from_bytes(&arr[4..20]).ok()?;
    let mut rdr = Cursor::new(&arr[20..24]);
    let t: f32 = rdr.read_u16::<BigEndian>().ok()?.try_into().ok()?;
    let sg: f32 = rdr.read_u16::<BigEndian>().ok()?.try_into().ok()?;

    Some(Tilt {
        uuid: u,
        sg: sg / 1000.0,
        t: (t - 32.0) / 1.8,
    })
}

pub fn filter_tilts(data: &Vec<u8>, tilt_uuids: HashMap<Uuid, String>) -> Option<Tilt> {
    println!("d: {:?}",data);
    let t = from(data)?;
    match tilt_uuids.contains_key(&t.uuid) {
        true => Some(t),
        _ => None,
    }
}

pub fn tilt_uuids() -> HashMap<Uuid, String> {
    let mut uuids = HashMap::new();
    uuids.insert(
        "A495BB80C5B14B44B5121370F02D74DE".parse().unwrap(),
        "pink".to_string(),
    );
    uuids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_tilt() {
        let pink_bytes: Vec<u8> = vec![
            76, 0, 2, 21, 164, 149, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
            222, 0, 67, 4, 4, 34,
        ];
        let other_bytes: Vec<u8> = vec![
            76, 0, 2, 21, 164, 159, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
            222, 0, 67, 4, 4, 34,
        ];

        assert!(&filter_tilts(&pink_bytes, tilt_uuids()).is_some());
        assert!(filter_tilts(&other_bytes, tilt_uuids()).is_none());
    }
}
