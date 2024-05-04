use futures::StreamExt;
mod tilt;

// 76 is required Apple iBeacon manufacturer Id
const MANUFACTURER_ID: u16 = 76;

#[tokio::main]
async fn main() -> bluer::Result<()> {
    let mut calibrate_g = 0.0;
    if let Some(g) = std::env::args().nth(1) {
        calibrate_g = g.parse::<f32>().unwrap();
    }
    let adapter = bluer::Session::new().await?.default_adapter().await?;
    adapter.set_powered(true).await?;
    let filter = bluer::DiscoveryFilter {
        transport: bluer::DiscoveryTransport::Le,
        ..Default::default()
    };
    adapter.set_discovery_filter(filter).await?;

    let mut device_events = adapter.discover_devices().await?;
    loop {
        tokio::select! {
            Some(bluer::AdapterEvent::DeviceAdded(addr) ) = device_events.next() => {
                if let Some(manufacturer_data) = adapter.device(addr)?.manufacturer_data().await?
                    && let Some(ibeacon_bytes) = manufacturer_data.get(&MANUFACTURER_ID) {
                    if let Ok(t) = tilt::Tilt::try_from(ibeacon_bytes) {
                        let gravity = t.gravity + calibrate_g;
                        println!(
                            "{{\"name\": \"{}\", \"gravity\": {}, \"temp\": {}}}",
                            t.name, gravity, t.temp
                        );
                        break;
                    }
                }
            },
            else => ()
        }
    }

    Ok(())
}
