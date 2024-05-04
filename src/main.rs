use std::error::Error;
mod tilt;
use btleplug::api::{Central, Manager as _, Peripheral as _};
use btleplug::platform::Manager;
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
        .get_matches();

    let calibrate_g = clap::value_t!(args.value_of("calibrate_g"), f32)?;

    println!("{}", scan_tilt(calibrate_g).await?);

    Ok(())
}

async fn scan_tilt(calibrate_g: f32) -> Result<String, TiltError> {
    let adapter = Manager::new()
        .await?
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or(TiltError::MissingAdapter)?;
    adapter.start_scan().await?;

    loop {
        for p in adapter.peripherals().await? {
            if let Some(k) = p.properties().await? {
                if let Ok(mut t) = Tilt::try_from(&k.manufacturer_data) {
                    t.gravity += calibrate_g;
                    adapter.stop_scan().await?;
                    return Ok(serde_json::to_string(&t).unwrap());
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum TiltError {
    #[error("BT")]
    BT(#[from] btleplug::Error),
    #[error("Missing adapter")]
    MissingAdapter,
}
