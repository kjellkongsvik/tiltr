use anyhow::{Context, Result};
use btleplug::api::{Central, Peripheral};
use btleplug::bluez::adapter::ConnectedAdapter;
use btleplug::bluez::manager::Manager;
use clap::{value_t, App, Arg};
use std::convert::{TryFrom, TryInto};
use std::thread;
use std::time::Duration;
mod tilt;
use crate::tilt::Tilt;

fn connect_adapter() -> Result<ConnectedAdapter> {
    let manager = Manager::new()?;

    let adapter = manager
        .adapters()?
        .into_iter()
        .next()
        .context("Device not found")?;

    Ok(adapter.connect()?)
}

fn scan_tilt(adapter: &ConnectedAdapter, timeout: usize) -> Option<Tilt> {
    for _ in 0..timeout {
        thread::sleep(Duration::from_secs(1));
        let found = adapter
            .peripherals()
            .into_iter()
            .filter_map(|p| p.properties().manufacturer_data)
            .filter_map(|v| v[..].try_into().ok())
            .find_map(|d| Tilt::try_from(&d).ok());
        if found.is_some() {
            return found;
        }
    }
    None
}

fn main() -> Result<()> {
    let args = App::new("Tilt logger")
        .arg(Arg::with_name("calibrate_sg").short("c").default_value("0"))
        .arg(Arg::with_name("timeout").short("t").default_value("1"))
        .get_matches();

    let timeout = value_t!(args.value_of("timeout"), usize)?;
    let calibrate = value_t!(args.value_of("calibrate_sg"), f32)?;

    let adapter = connect_adapter()?;
    adapter.start_scan()?;

    if let Some(mut t) = scan_tilt(&adapter, timeout) {
        t.gravity += calibrate;
        println!("{}", serde_json::to_string(&t)?);
    }

    adapter.stop_scan()?;

    Ok(())
}
