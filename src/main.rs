extern crate rand;
extern crate rumble;
mod lib;

use lib::filter_tilts;
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

fn scan_tilt() -> Result<Option<Vec<u8>>, rumble::Error> {
    let adapter = connect_adapter(0)?;
    adapter.start_scan()?;
    'outer: loop {
        thread::sleep(Duration::from_secs(1));
        for c in adapter.peripherals().into_iter() {
            let d = c.properties().manufacturer_data;
            match filter_tilts(&d) {
                Some(t) => println!("{:#?}", t),
                _ => (),
            }
        }
    }
}

pub fn main() -> Result<(), rumble::Error> {
    println!("{:?}", scan_tilt()?);

    Ok(())
}
