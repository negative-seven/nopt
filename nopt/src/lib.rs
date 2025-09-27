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

            let mmap = Compiler::new(true).compile(&mut self.nes);

            unsafe { std::mem::transmute(ManuallyDrop::new(mmap).as_ptr()) }
        };

        let function = if (0x8000..0xfffe).contains(&pc) {
            *self
                .prg_rom_functions
                .get_mut(usize::from(pc) & (prg_rom_len - 1))
                .unwrap()
                .get_or_insert_with(compile)
        } else {
            compile()
        };

        trace!("running with pc: 0x{pc:04x}");
        unsafe {
            function();
        }
    }
}
