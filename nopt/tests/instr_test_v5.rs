use nopt::{Nopt, Rom};
use std::ffi::OsStr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[test]
fn instr_test_v5() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("nopt", tracing_subscriber::filter::LevelFilter::TRACE)
                .with_target(
                    "instr_test_v5",
                    tracing_subscriber::filter::LevelFilter::TRACE,
                ),
        )
        .init();

    for rom_dir_entry in std::fs::read_dir("tests/roms/instr_test_v5").unwrap() {
        let rom_dir_entry = rom_dir_entry.unwrap();
        if !rom_dir_entry
            .metadata()
            .is_ok_and(|metadata| metadata.is_file())
        {
            continue;
        }
        if rom_dir_entry.file_name() == OsStr::new(".gitignore") {
            continue;
        }
        info!("running test: {}", rom_dir_entry.file_name().display());

        let rom = Rom::from_bytes_with_header(std::fs::read(rom_dir_entry.path()).unwrap());

        let mut nopt = Nopt::new(rom);

        let result_code;
        loop {
            unsafe {
                nopt.run();
            }

            let data_is_valid = (0x6001..0x6004)
                .map(|address| nopt.nes().peek(address))
                .collect::<Vec<_>>()
                == [0xde, 0xb0, 0x61];
            if !data_is_valid {
                continue;
            }

            let status = nopt.nes().peek(0x6000);
            match status {
                0x00..0x80 => {
                    result_code = status;
                    break;
                }
                0x80 => (),
                0x81 => unimplemented!("reset during instr_test_v5 test"),
                0x82.. => unreachable!(),
            }
        }

        let message = (0x6004..)
            .map(|address| nopt.nes().peek(address))
            .take_while(|value| *value != 0)
            .map(|value| if value.is_ascii() { value as char } else { '?' })
            .collect::<String>();
        for line in message.lines() {
            info!("test output: {line:?}");
        }

        if result_code == 0 {
            continue;
        }

        let first_failed_opcode = u8::from_str_radix(&message[..2], 16).unwrap();
        if [0x03, 0x07, 0x13, 0x0f, 0x17, 0x1f, 0xeb].contains(&first_failed_opcode) {
            info!(
                "assuming success based on the first failed opcode being unofficial: 0x{:02x}",
                first_failed_opcode
            );
            continue;
        }

        panic!("result code: {result_code}");
    }
}
