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

fn to_array(i: &Vec<u8>) -> Option<[u8; 25]> {
    (i[..]).try_into().ok()
}

fn from(item: &Option<Vec<u8>>) -> Option<Tilt> {
    let arr = match item {
        Some(x) => to_array(&x)?,
        None => return None,
    };
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

pub fn filter_tilts(
    data: &Option<std::vec::Vec<u8>>,
    tilt_uuids: HashMap<String, Uuid>,
) -> Option<Tilt> {
    let t = from(data)?;
    match tilt_uuids.values().any(|x| x == &t.uuid) {
        true => Some(t),
        _ => None,
    }
}

pub fn tilt_uuids() -> HashMap<String, Uuid> {
    let mut uuids = HashMap::new();
    uuids.insert(
        "pink".to_string(),
        "A495BB80C5B14B44B5121370F02D74DE".parse().unwrap(),
    );
    uuids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_tilt() {
        let pink_bytes: Option<Vec<u8>> = Some(vec![
            76, 0, 2, 21, 164, 149, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
            222, 0, 67, 4, 4, 34,
        ]);
        let other_bytes: Option<Vec<u8>> = Some(vec![
            76, 0, 2, 21, 164, 159, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116,
            222, 0, 67, 4, 4, 34,
        ]);

        assert_eq!(
            &filter_tilts(&pink_bytes, tilt_uuids()).unwrap().uuid,
            tilt_uuids().get("pink").unwrap()
        );
        assert!(filter_tilts(&other_bytes, tilt_uuids()).is_none());
    }
}
