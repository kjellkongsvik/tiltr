extern crate rand;
extern crate rumble;

use rumble::api::{Central, Peripheral};
use rumble::bluez::adapter::ConnectedAdapter;
use rumble::bluez::manager::Manager;
use std::thread;
use std::time::Duration;

fn connect_adapter(dev: usize) -> Result<ConnectedAdapter, rumble::Error> {
    let manager = Manager::new()?;

    let adapters = manager.adapters()?;
    let mut adapter = adapters.into_iter().nth(dev).unwrap();

    adapter = manager.down(&adapter)?;
    adapter = manager.up(&adapter)?;

    adapter.connect()
}

pub fn main() -> Result<(), rumble::Error> {
    let central = connect_adapter(0)?;
    // start scanning for devices
    central.start_scan().unwrap();
    thread::sleep(Duration::from_secs(10));

    // find the device we're interested in
    for c in central.peripherals().into_iter() {
        // c.discover_characteristics().unwrap();
        let p = c.properties();
        // if p.tx_power_level != None {
        println!("{:?}", c.properties());
        // }
    }
    Ok(())
}
