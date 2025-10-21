use nopt::{Nopt, Rom};
use regex::Regex;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[test]
#[ignore = "unimplemented"]
fn ppu_read_buffer() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("nopt", tracing_subscriber::filter::LevelFilter::TRACE)
                .with_target(
                    "ppu_read_buffer",
                    tracing_subscriber::filter::LevelFilter::TRACE,
                ),
        )
        .init();

    let rom =
        Rom::from_bytes_with_header(std::fs::read("tests/roms/test_ppu_read_buffer.nes").unwrap());

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

    let ansi_color_codes_regex = Regex::new(r"\x1b\[[\d;]*[^\d;]").unwrap();
    let message = (0x6004..)
        .map(|address| nopt.nes().peek(address))
        .take_while(|value| *value != 0)
        .map(|value| if value.is_ascii() { value as char } else { '?' })
        .collect::<String>();
    let message = ansi_color_codes_regex.replace_all(&message, "");
    for line in message.lines() {
        info!("test output: {line:?}");
    }

    if result_code == 0 {
        return;
    }

    panic!("result code: {result_code}");
}
