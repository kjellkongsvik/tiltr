extern crate rand;
extern crate rumble;
mod brewfather;
mod lib;

use clap::{App, Arg};
use lib::tilt_uuids;
use lib::{Tilt, TiltError};
use rumble::api::{Central, Peripheral};
use rumble::bluez::adapter::ConnectedAdapter;
use rumble::bluez::manager::Manager;
use serde_json::json;
use std::convert::TryFrom;
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

fn scan_tilt(adapter: &ConnectedAdapter, timeout: i32) -> Result<Tilt, TiltError> {
    let uuids = tilt_uuids();
    for _ in 0..timeout {
        thread::sleep(Duration::from_secs(1));
        let tilts: Vec<Tilt> = adapter
            .peripherals()
            .into_iter()
            .filter_map(|p| p.properties().manufacturer_data)
            .filter_map(|data| Tilt::try_from(&data).ok())
            .filter(|tilt| uuids.values().any(|u| u == &tilt.name))
            .collect();
        if let Some(t) = tilts.first() {
            return Ok(t.clone());
        }
    }
    Err(TiltError::NoTiltResponse)
}

pub fn main() -> Result<(), TiltError> {
    let args = App::new("Tilt logger")
        .arg(
            Arg::with_name("url")
                .help("if set posts there")
                .value_name("BREWFATHER_URL")
                .required(false),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .help("Number of seconds to wait for a tilt")
                .value_name("TIMEOUT_SECONDS")
                .required(false),
        )
        .get_matches();

    let url = args.value_of("url").unwrap_or("");
    let timeout: i32 = args.value_of("timeout").unwrap_or("10").parse().unwrap();

    let adapter = connect_adapter(0).unwrap();
    adapter.start_scan().unwrap();
    let mut ts = scan_tilt(&adapter, timeout)?;
    ts.name = "pink".to_string();
    println!("{:?}", json!(&ts));

    if !url.is_empty() {
        match brewfather::post(url, &json!(&ts)) {
            Ok(r) => println!("{:?}", r),
            Err(e) => println!("{:?}", e),
        };
    }
    Ok(())
}
