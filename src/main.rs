use anyhow::{Context, Result};
use btleplug::api::{Central, Manager as _, Peripheral as _};
use btleplug::platform::{Adapter, Manager};
use clap::{value_t, App, Arg};
use std::convert::TryFrom;
use std::error::Error;
use std::thread;
use std::time::Duration;
mod tilt;

async fn connect_adapter() -> Result<Adapter> {
    let manager = Manager::new().await?;

    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .context("Blutooth adapter not found")?;
    Ok(adapter)
}

async fn scan_tilt(adapter: &Adapter, timeout: usize) -> Option<tilt::Tilt> {
    for _ in 0..timeout {
        thread::sleep(Duration::from_secs(1));
        for p in adapter.peripherals().await.unwrap() {
            let k = p.properties().await.unwrap().unwrap();
            if let Ok(t) = tilt::Tilt::try_from(&k.manufacturer_data) {
                return Some(t);
            }
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new("Tilt logger")
        .arg(Arg::with_name("calibrate_sg").short("c").default_value("0"))
        .arg(Arg::with_name("timeout").short("t").default_value("1"))
        .get_matches();

    let timeout = value_t!(args.value_of("timeout"), usize)?;
    let calibrate = value_t!(args.value_of("calibrate_sg"), f32)?;

    let adapter = connect_adapter().await?;
    adapter.start_scan().await?;

    if let Some(mut t) = scan_tilt(&adapter, timeout).await {
        t.gravity += calibrate;
        println!("{}", serde_json::to_string(&t)?);
    }

    adapter.stop_scan().await?;

    Ok(())
}
