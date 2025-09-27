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
    functions: Box<[Option<unsafe extern "C" fn()>; 0x10000]>,
}

impl Nopt {
    #[must_use]
    pub fn new(rom: Rom) -> Self {
        Self {
            nes: Nes::new(rom),
            functions: vec![None; 0x10000].into_boxed_slice().try_into().unwrap(),
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
        let pc = self.nes.cpu.pc;

        let function = self
            .functions
            .get_mut(usize::from(pc))
            .unwrap()
            .get_or_insert_with(|| {
                trace!("compiling function at 0x{pc:04x}");

                let mmap = Compiler::new(true).compile(&mut self.nes);

                unsafe { std::mem::transmute(ManuallyDrop::new(mmap).as_ptr()) }
            });

        trace!("running with pc: 0x{pc:04x}");
        unsafe {
            function();
        }
    }
}
