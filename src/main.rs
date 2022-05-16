use std::error::Error;
mod tilt;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use std::convert::TryFrom;
use tilt::Tilt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = clap::Command::new("tiltr")
        .version(clap::crate_version!())
        .arg(clap::Arg::new("calibrate_g").short('c').default_value("0"))
        .arg(clap::Arg::new("timeout").short('t').default_value("1"))
        .get_matches();

    let calibrate_g = args.value_of_t("calibrate_g")?;
    let timeout = args.value_of_t("timeout")?;

    println!("{}", scan_tilt(calibrate_g, timeout).await?);

    Ok(())
}

async fn search(central: &Adapter) -> Result<Tilt, TiltError> {
    loop {
        for p in central.peripherals().await? {
            if let Some(k) = p.properties().await? {
                if let Ok(tilt) = Tilt::try_from(&k.manufacturer_data) {
                    return Ok(tilt);
                }
            }
        }
    }
}

async fn scan_tilt(calibrate_g: f32, timeout: u64) -> Result<String, TiltError> {
    let central = Manager::new()
        .await?
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or(TiltError::MissingAdapter)?;
    central
        .start_scan(ScanFilter {
            services: tilt::tilt_uuids().into_keys().collect(),
        })
        .await?;

    let tilt = match tokio::time::timeout(
        std::time::Duration::from_secs(timeout),
        search(&central),
    )
    .await
    {
        Ok(Ok(mut t)) => {
            t.gravity += calibrate_g;
            Ok(serde_json::to_string(&t).unwrap())
        }
        _ => Err(TiltError::TiltNotFound),
    };
    central.stop_scan().await?;
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
