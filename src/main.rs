extern crate rand;
extern crate rumble;
mod lib;

use lib::filter_tilts;
use lib::tilt_uuids;
use lib::Tilt;
use rumble::api::{Central, Peripheral};
use rumble::bluez::adapter::ConnectedAdapter;
use rumble::bluez::manager::Manager;
use std::thread;
use std::time::Duration;

fn connect_adapter(dev: usize) -> Result<ConnectedAdapter, rumble::Error> {
    let manager = Manager::new()?;

    let adapters = manager.adapters()?;
    let mut adapter = match adapters.into_iter().nth(dev) {
        Some(x) => x,
        None => return Err(rumble::Error::DeviceNotFound),
    };

    adapter = manager.down(&adapter)?;
    adapter = manager.up(&adapter)?;

    adapter.connect()
}

fn scan_tilt(adapter: &ConnectedAdapter) -> Vec<Tilt> {
    for _ in 0..20 {
        thread::sleep(Duration::from_secs(1));
        let tilts: Vec<Tilt> = adapter
            .peripherals()
            .into_iter()
            .filter_map(|p| p.properties().manufacturer_data)
            .filter_map(|d| filter_tilts(&d, tilt_uuids()))
            .collect();
        println!("X: {:?}", tilts);
        if tilts.len() > 0 {
            return tilts;
        };
    }
    vec![]
}

pub fn main() {
    let adapter = connect_adapter(0).unwrap();
    adapter.start_scan().unwrap();
    println!("{:?}", scan_tilt(&adapter));
}
