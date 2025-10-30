mod compiler;
mod nes_assembly;

use crate::compiler::{Compiler, frontend::nes::Nes};
pub use compiler::frontend::nes::cartridge;
use std::mem::ManuallyDrop;
use tracing::trace;

pub struct Nopt<Cartridge: cartridge::Cartridge> {
    nes: Nes<Cartridge>,
    prg_rom_functions: Vec<Option<unsafe extern "C" fn()>>,
}

impl<Cartridge: cartridge::Cartridge> Nopt<Cartridge> {
    #[must_use]
    pub fn new(cartridge: Cartridge) -> Self {
        let nes = Nes::new(cartridge);
        let functions = vec![None; 0x8000];
        Self {
            nes,
            prg_rom_functions: functions,
        }
    }

    #[must_use]
    pub fn nes(&self) -> &Nes<Cartridge> {
        &self.nes
    }

    pub fn nes_mut(&mut self) -> &mut Nes<Cartridge> {
        &mut self.nes
    }

    /// # Safety
    ///
    /// The safety of this function is conditional on the safety of the
    /// underlying backend.
    pub unsafe fn run(&mut self) {
        let pc = self.nes().cpu.pc;

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
                let prg_rom_functions_len = self.prg_rom_functions.len();
                self.prg_rom_functions
                    .get_mut(usize::from(pc) & (prg_rom_functions_len - 1))
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
