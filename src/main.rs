use clap::{value_t, App, Arg};
use reqwest::Client;
use rumble::api::{Central, Peripheral};
use rumble::bluez::adapter::ConnectedAdapter;
use rumble::bluez::manager::Manager;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::thread;
use std::time::Duration;
use tilt::{t_data, Tilt};

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

fn scan_tilt(timeout: u32, n: usize) -> Vec<Tilt> {
    let adapter = connect_adapter(0).expect("connecting adapter");
    adapter.start_scan().expect("start scan");

    let mut found = HashMap::new();

    for _ in 0..timeout {
        thread::sleep(Duration::from_secs(1));
        adapter
            .peripherals()
            .into_iter()
            .filter_map(|p| p.properties().manufacturer_data)
            .filter_map(|v| t_data(&v).ok())
            .filter_map(|data| Tilt::try_from(&data).ok())
            .fold((), |_, v| {
                found.entry(v.name.clone()).or_insert(v);
            });
        if found.len() == n {
            break;
        }
    }

    adapter.stop_scan().expect("stop scan");
    found.drain().map(|(_, v)| v).collect()
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = App::new("Tilt logger")
        // TODO device#
        .arg(Arg::with_name("url").short("u"))
        .arg(Arg::with_name("num").short("n").default_value("1"))
        .arg(Arg::with_name("timeout").short("t").default_value("10"))
        .get_matches();

    let timeout = value_t!(args.value_of("timeout"), u32).unwrap_or_else(|e| e.exit());
    let num = value_t!(args.value_of("num"), usize).unwrap_or_else(|e| e.exit());

    let tilts = scan_tilt(timeout, num);

    println!("{:?}", &tilts);

    if let Some(url) = args.value_of("url") {
        let client = Client::new();
        for t in tilts {
            client.post(url).json(&t).send().await?;
        }
    }
    Ok(())
}
