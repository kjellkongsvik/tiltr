use std::error::Error;
mod tilt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = clap::App::new("Tilt logger")
        .arg(
            clap::Arg::with_name("calibrate_sg")
                .short("c")
                .default_value("0"),
        )
        .arg(
            clap::Arg::with_name("timeout")
                .short("t")
                .default_value("1"),
        )
        .get_matches();

    let calibrate = clap::value_t!(args.value_of("calibrate_sg"), f32)?;
    let timeout = clap::value_t!(args.value_of("timeout"), usize)?;

    println!("{}", tilt::scan_tilt(calibrate, timeout).await?);

    Ok(())
}
