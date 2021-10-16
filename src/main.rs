use std::error::Error;
mod tilt;
use btleplug::api::{Central, Manager as _, Peripheral as _};
use btleplug::platform::{Adapter, Manager};
use std::convert::TryFrom;
use tilt::Tilt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = clap::App::new("tiltr")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::with_name("calibrate_g")
                .short("c")
                .default_value("0"),
        )
        .arg(
            clap::Arg::with_name("timeout")
                .short("t")
                .default_value("1"),
        )
        .get_matches();

    let calibrate_g = clap::value_t!(args.value_of("calibrate_g"), f32)?;
    let timeout = clap::value_t!(args.value_of("timeout"), u64)?;

    println!("{}", scan_tilt(calibrate_g, timeout).await?);

    Ok(())
}

async fn search(adapter: &Adapter) -> Result<Tilt, TiltError> {
    loop {
        for p in adapter.peripherals().await? {
            if let Some(k) = p.properties().await? {
                if let Ok(tilt) = Tilt::try_from(&k.manufacturer_data) {
                    return Ok(tilt);
                }
            }
        }
    }
}

async fn scan_tilt(calibrate_g: f32, timeout: u64) -> Result<String, TiltError> {
    let adapter = Manager::new()
        .await?
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or(TiltError::MissingAdapter)?;
    adapter.start_scan().await?;

    let tilt = match tokio::time::timeout(
        std::time::Duration::from_secs(timeout),
        search(&adapter),
    )
    .await
    {
        Ok(Ok(mut t)) => {
            t.gravity += calibrate_g;
            Ok(serde_json::to_string(&t).unwrap())
        }
        _ => Err(TiltError::TiltNotFound),
    };
    adapter.stop_scan().await?;
    tilt
}

#[derive(Debug, thiserror::Error)]
enum TiltError {
    #[error("Tilt not found")]
    TiltNotFound,
    #[error("BT")]
    BT(#[from] btleplug::Error),
    #[error("Missing adapter")]
    MissingAdapter,
}
