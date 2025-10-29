use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("nopt", tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();

    let Some(rom_filepath) = std::env::args().nth(1) else {
        panic!("missing argument: rom filepath");
    };
    let rom: Vec<u8> = std::fs::read(rom_filepath).unwrap();

    let mut runtime = nopt::Nopt::new(nopt::cartridge::from_bytes_with_header(&rom));
    unsafe {
        loop {
            runtime.run();
        }
    }
}
