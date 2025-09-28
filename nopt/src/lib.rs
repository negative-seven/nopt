mod compiler;
mod nes;
mod nes_assembly;
mod rom;

use crate::{compiler::Compiler, nes::Nes};
pub use rom::Rom;
use std::mem::ManuallyDrop;
use tracing::trace;

pub struct Nopt {
    nes: Nes,
    prg_rom_functions: Vec<Option<unsafe extern "C" fn()>>,
}

impl Nopt {
    #[must_use]
    pub fn new(rom: Rom) -> Self {
        let nes = Nes::new(rom);
        let functions = vec![None; nes.rom.prg_rom().len()];
        Self {
            nes,
            prg_rom_functions: functions,
        }
    }

    #[must_use]
    pub fn nes(&self) -> &Nes {
        &self.nes
    }

    pub fn nes_mut(&mut self) -> &mut Nes {
        &mut self.nes
    }

    /// # Safety
    ///
    /// The safety of this function is conditional on the safety of the
    /// underlying backend.
    pub unsafe fn run(&mut self) {
        let pc = self.nes().cpu.pc;
        let prg_rom_len = self.nes().rom.prg_rom().len();

        let mut compile = || {
            trace!("compiling function at 0x{pc:04x}");

            let (mmap, is_prg_rom_only) = Compiler::new(true).compile(&mut self.nes);

            (
                unsafe {
                    std::mem::transmute::<*const u8, unsafe extern "C" fn()>(
                        ManuallyDrop::new(mmap).as_ptr(),
                    )
                },
                is_prg_rom_only,
            )
        };

        let function = {
            let mut dummy_entry = None;
            let entry = if pc >= 0x8000 {
                self.prg_rom_functions
                    .get_mut(usize::from(pc) & (prg_rom_len - 1))
                    .unwrap()
            } else {
                &mut dummy_entry
            };

            if let Some(function) = entry.as_mut() {
                *function
            } else {
                let (function, is_prg_rom_only) = compile();
                if is_prg_rom_only {
                    *entry = Some(function);
                }
                function
            }
        };

        trace!("running with pc: 0x{pc:04x}");
        unsafe {
            function();
        }
    }
}
