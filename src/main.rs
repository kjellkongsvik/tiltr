use clap::{value_t, App, Arg};
use rumble::api::{Central, Peripheral};
use rumble::bluez::adapter::ConnectedAdapter;
use rumble::bluez::manager::Manager;
use std::convert::{TryFrom, TryInto};
use std::thread;
use std::time::Duration;
use tilt::Tilt;

fn connect_adapter(dev: usize) -> Result<ConnectedAdapter, rumble::Error> {
    let manager = Manager::new()?;

    let adapter = manager
        .adapters()?
        .into_iter()
        .nth(dev)
        .ok_or(rumble::Error::DeviceNotFound)?;

    manager.down(&adapter)?;
    manager.up(&adapter)?;

    adapter.connect()
}

fn scan_tilt(adapter: &ConnectedAdapter, timeout: usize) -> Option<Tilt> {
    for _ in 0..timeout {
        thread::sleep(Duration::from_secs(1));
        let found = adapter
            .peripherals()
            .into_iter()
            .filter_map(|p| p.properties().manufacturer_data)
            .filter_map(|v| v[..].try_into().ok())
            .filter_map(|d| Tilt::try_from(&d).ok())
            .next();
        if let Some(_) = found {
            return found;
        }
    }
    None
}

fn main() -> Result<(), rumble::Error> {
    let args = App::new("Tilt logger")
        .arg(Arg::with_name("device").short("d").default_value("0"))
        .arg(Arg::with_name("timeout").short("t").default_value("1"))
        .get_matches();

    let device = value_t!(args.value_of("device"), usize).unwrap_or_else(|e| e.exit());
    let timeout = value_t!(args.value_of("timeout"), usize).unwrap_or_else(|e| e.exit());

    let adapter = connect_adapter(device)?;
    adapter.start_scan()?;

    if let Some(t) = scan_tilt(&adapter, timeout) {
        println!("{}", &t);
    }

    adapter.stop_scan()?;

    Ok(())
}
