use nopt::{Cartridge, Nopt};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[test]
fn nestest() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("nopt", tracing_subscriber::filter::LevelFilter::TRACE)
                .with_target("nestest", tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();

    let cartridge = Cartridge::from_bytes_with_header(&{
        let mut bytes = std::fs::read("tests/roms/nestest/nestest.nes").unwrap();

        // patch reset vector to enable the automated test runner
        bytes[0x400c..0x400e].copy_from_slice(&0xc000u16.to_le_bytes());

        bytes
    });
    let mut nopt = Nopt::new(cartridge);

    // todo: streamline this setup
    nopt.nes_mut().cpu.p = 0x24;
    nopt.nes_mut().cpu.s = 0xfd;

    let log = std::fs::read_to_string("tests/roms/nestest/nestest.log").unwrap();
    for log_line in log.lines() {
        if log_line.contains('*') {
            // unofficial instruction
            break;
        }

        let nopt_log_line = {
            let pc = nopt.nes().cpu.pc;
            let a = nopt.nes().cpu.a;
            let x = nopt.nes().cpu.x;
            let y = nopt.nes().cpu.y;
            let s = nopt.nes().cpu.s;
            let p = nopt.nes().cpu.p;
            format!(
                "{pc:04X}  __ __ __  ______________________________  A:{a:02X} X:{x:02X} Y:{y:02X} P:{p:02X} SP:{s:02X} PPU:___,___ CYC:_____"
            )
        };

        let comparison_result = nopt_log_line
            .chars()
            .zip(log_line.chars())
            .all(|(a, b)| a == '_' || a == b);
        if !comparison_result {
            // use `assert_eq` for pretty print
            assert_eq!(nopt_log_line, log_line);
        }

        unsafe {
            nopt.run();
        }
    }
}
