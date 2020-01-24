extern crate rumble;

use clap::{value_t, App, Arg};
use reqwest::Client;
use reqwest::Error;
use rumble::api::{Central, Peripheral};
use rumble::bluez::adapter::ConnectedAdapter;
use rumble::bluez::manager::Manager;
use std::convert::TryFrom;
use std::thread;
use std::time::Duration;
use tilt::{t_data, Tilt};

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

fn scan_tilt(timeout: u32) -> Vec<Tilt> {
    let adapter = connect_adapter(0).expect("connecting adapter");
    adapter.start_scan().expect("start scan");

    for _ in 0..timeout {
        thread::sleep(Duration::from_secs(1));
        let tilts: Vec<Tilt> = adapter
            .peripherals()
            .into_iter()
            .filter_map(|p| p.properties().manufacturer_data)
            .filter_map(|v| t_data(&v).ok())
            .filter_map(|data| Tilt::try_from(&data).ok())
            .collect();
        return tilts;
    }

    adapter.stop_scan().expect("stop scan");
    Vec::new()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = App::new("Tilt logger")
        .arg(Arg::with_name("url").short("u"))
        .arg(Arg::with_name("timeout").short("t").default_value("10"))
        .get_matches();

    let timeout = value_t!(args.value_of("timeout"), u32).unwrap_or_else(|e| e.exit());
    let tilts = scan_tilt(timeout);

    println!("{:?}", &tilts);

    if let Some(url) = args.value_of("url") {
        let client = Client::new();
        for t in tilts {
            client.post(url).json(&t).send().await?;
        }
    }
    Ok(())
}
