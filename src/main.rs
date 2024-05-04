mod tilt;
use btleplug::api::{Central, Manager as _, Peripheral as _};
use btleplug::platform::Manager;
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> btleplug::Result<()> {
    let mut calibrate_g: f32 = 0.0;
    if let Some(g) = std::env::args().nth(1) {
        calibrate_g = g.parse().unwrap();
    }

    let adapter = Manager::new()
        .await?
        .adapters()
        .await?
        .into_iter()
        .next()
        .unwrap();
    adapter.start_scan().await?;

    loop {
        for p in adapter.peripherals().await? {
            if let Some(k) = p.properties().await? {
                if let Ok(mut t) = tilt::Tilt::try_from(&k.manufacturer_data) {
                    t.gravity += calibrate_g;
                    adapter.stop_scan().await?;
                    println!("{}", serde_json::to_string(&t).unwrap());

                    return Ok(());
                }
            }
        }
    }
}
