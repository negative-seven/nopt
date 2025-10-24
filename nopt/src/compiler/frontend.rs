mod r#abstract;
mod instruction_decoder;

use crate::{
    compiler::{
        frontend::r#abstract::Compiler,
        ir::{
            BasicBlock, CpuFlag, CpuRegister, Definition1, Definition8, Definition16, Destination1,
            Destination8, Destination16, Function, Instruction, Jump, Variable1, Variable8,
            Variable16,
        },
    },
    nes::Nes,
};
use std::{cell::RefCell, rc::Rc, sync::atomic::AtomicUsize};

pub(super) fn compile_instruction(nes: &mut Nes, address: u16) -> (Function, bool) {
    let (nes_instruction, is_prg_rom_only) = instruction_decoder::decode_instruction(nes, address);

    let basic_block = Rc::new(RefCell::new(BasicBlock::new(Rc::new(AtomicUsize::new(0)))));
    Compiler {
        visitor: CompilerVisitor {
            current_block: Rc::clone(&basic_block),
        },
        cpu_instruction: nes_instruction,
    }
    .transpile();

    (Function { basic_block }, is_prg_rom_only)
}

pub(crate) struct CompilerVisitor {
    current_block: Rc<RefCell<BasicBlock>>,
}

impl CompilerVisitor {
    pub(crate) fn cpu_c(&self) -> Destination1 {
        Destination1::CpuFlag(CpuFlag::C)
    }

    pub(crate) fn cpu_z(&self) -> Destination1 {
        Destination1::CpuFlag(CpuFlag::Z)
    }

    pub(crate) fn cpu_i(&self) -> Destination1 {
        Destination1::CpuFlag(CpuFlag::I)
    }

    pub(crate) fn cpu_d(&self) -> Destination1 {
        Destination1::CpuFlag(CpuFlag::D)
    }

    pub(crate) fn cpu_b(&self) -> Destination1 {
        Destination1::CpuFlag(CpuFlag::B)
    }

    pub(crate) fn cpu_unused_flag(&self) -> Destination1 {
        Destination1::CpuFlag(CpuFlag::Unused)
    }

    pub(crate) fn cpu_v(&self) -> Destination1 {
        Destination1::CpuFlag(CpuFlag::V)
    }

    pub(crate) fn cpu_n(&self) -> Destination1 {
        Destination1::CpuFlag(CpuFlag::N)
    }

    pub(crate) fn cpu_a(&self) -> Destination8 {
        Destination8::CpuRegister(CpuRegister::A)
    }

    pub(crate) fn cpu_x(&self) -> Destination8 {
        Destination8::CpuRegister(CpuRegister::X)
    }

    pub(crate) fn cpu_y(&self) -> Destination8 {
        Destination8::CpuRegister(CpuRegister::Y)
    }

    pub(crate) fn cpu_s(&self) -> Destination8 {
        Destination8::CpuRegister(CpuRegister::S)
    }

    pub(crate) fn cpu_p(&self) -> Destination8 {
        Destination8::CpuRegister(CpuRegister::P)
    }

    fn cpu_pc(&mut self) -> Variable16 {
        self.define_16(Definition16::Pc)
    }

    fn ppu_current_address(&mut self) -> Variable16 {
        self.define_16(Definition16::PpuCurrentAddress)
    }

    fn cpu_ram(&mut self, address: Variable16) -> Variable8 {
        self.define_8(Definition8::CpuRam(address))
    }

    fn prg_ram(&mut self, address: Variable16) -> Variable8 {
        self.define_8(Definition8::PrgRam(address))
    }

    fn ppu_ram(&mut self, address: Variable16) -> Variable8 {
        self.define_8(Definition8::PpuRam(address))
    }

    fn rom(&mut self, address: Variable16) -> Variable8 {
        self.define_8(Definition8::Rom(address))
    }

    fn get_bit(&mut self, operand: Variable8, index: u8) -> Variable1 {
        self.define_1(Definition1::U8Bit { operand, index })
    }

    fn not(&mut self, operand: Variable1) -> Variable1 {
        self.define_1(Definition1::Not(operand))
    }

    fn is_zero(&mut self, operand: Variable8) -> Variable1 {
        self.define_1(Definition1::EqualToZero(operand))
    }

    fn is_negative(&mut self, operand: Variable8) -> Variable1 {
        self.define_1(Definition1::Negative(operand))
    }

    fn rotate_left(&mut self, operand: Variable8, operand_carry: Variable1) -> Variable8 {
        self.define_8(Definition8::RotateLeft {
            operand,
            operand_carry,
        })
    }

    fn rotate_right(&mut self, operand: Variable8, operand_carry: Variable1) -> Variable8 {
        self.define_8(Definition8::RotateRight {
            operand,
            operand_carry,
        })
    }

    fn low_byte(&mut self, operand: Variable16) -> Variable8 {
        self.define_8(Definition8::LowByte(operand))
    }

    fn high_byte(&mut self, operand: Variable16) -> Variable8 {
        self.define_8(Definition8::HighByte(operand))
    }

    fn less_than_or_equal(&mut self, operand_0: Variable16, operand_1: Variable16) -> Variable1 {
        self.define_1(Definition1::LessThanOrEqual16(operand_0, operand_1))
    }

    fn select(
        &mut self,
        condition: Variable1,
        result_if_true: Variable16,
        result_if_false: Variable16,
    ) -> Variable16 {
        self.define_16(Definition16::Select {
            condition,
            result_if_true,
            result_if_false,
        })
    }

    fn concatenate(&mut self, high: Variable8, low: Variable8) -> Variable16 {
        self.define_16(Definition16::FromU8s { high, low })
    }

    pub(crate) fn and(&mut self, operand_0: Variable8, operand_1: Variable8) -> Variable8 {
        self.define_8(Definition8::And(operand_0, operand_1))
    }

    pub(crate) fn add_u8(
        &mut self,
        operand_0: Variable8,
        operand_1: Variable8,
        operand_carry: Variable1,
    ) -> Variable8 {
        self.define_8(Definition8::Sum {
            operand_0,
            operand_1,
            operand_carry,
        })
    }

    pub(crate) fn add_u8_carry(
        &mut self,
        operand_0: Variable8,
        operand_1: Variable8,
        operand_carry: Variable1,
    ) -> Variable1 {
        self.define_1(Definition1::SumCarry {
            operand_0,
            operand_1,
            operand_carry,
        })
    }

    pub(crate) fn add_u8_overflow(
        &mut self,
        operand_0: Variable8,
        operand_1: Variable8,
        operand_carry: Variable1,
    ) -> Variable1 {
        self.define_1(Definition1::SumOverflow {
            operand_0,
            operand_1,
            operand_carry,
        })
    }

    fn add_u16(&mut self, operand_0: Variable16, operand_1: Variable16) -> Variable16 {
        self.define_16(Definition16::Sum {
            operand_0,
            operand_1,
        })
    }

    fn sub(
        &mut self,
        operand_0: Variable8,
        operand_1: Variable8,
        operand_borrow: Variable1,
    ) -> Variable8 {
        self.define_8(Definition8::Difference {
            operand_0,
            operand_1,
            operand_borrow,
        })
    }

    fn sub_borrow(
        &mut self,
        operand_0: Variable8,
        operand_1: Variable8,
        operand_borrow: Variable1,
    ) -> Variable1 {
        self.define_1(Definition1::DifferenceBorrow {
            operand_0,
            operand_1,
            operand_borrow,
        })
    }

    fn sub_overflow(
        &mut self,
        operand_0: Variable8,
        operand_1: Variable8,
        operand_borrow: Variable1,
    ) -> Variable1 {
        self.define_1(Definition1::DifferenceOverflow {
            operand_0,
            operand_1,
            operand_borrow,
        })
    }

    pub(crate) fn define_1(&mut self, definition: impl Into<Definition1>) -> Variable1 {
        self.current_block.borrow_mut().define_1(definition.into())
    }

    pub(crate) fn define_8(&mut self, definition: impl Into<Definition8>) -> Variable8 {
        self.current_block.borrow_mut().define_8(definition.into())
    }

    pub(crate) fn define_16(&mut self, definition: impl Into<Definition16>) -> Variable16 {
        self.current_block.borrow_mut().define_16(definition.into())
    }

    pub(crate) fn store_1(&mut self, destination: impl Into<Destination1>, register: Variable1) {
        self.current_block
            .borrow_mut()
            .instructions
            .push(Instruction::Store1 {
                variable: register,
                destination: destination.into(),
            });
    }

    pub(crate) fn store_8(&mut self, destination: impl Into<Destination8>, register: Variable8) {
        self.current_block
            .borrow_mut()
            .instructions
            .push(Instruction::Store8 {
                variable: register,
                destination: destination.into(),
            });
    }

    pub(crate) fn store_16(&mut self, destination: impl Into<Destination16>, register: Variable16) {
        self.current_block
            .borrow_mut()
            .instructions
            .push(Instruction::Store16 {
                variable: register,
                destination: destination.into(),
            });
    }

    pub(super) fn if_else(
        &mut self,
        condition: Variable1,
        populate_true_block: impl Fn(&mut CompilerVisitor),
        populate_false_block: impl Fn(&mut CompilerVisitor),
    ) {
        let unused_variable = self.define_8(0);
        self.if_else_with_result(
            condition,
            |block| {
                populate_true_block(block);
                unused_variable
            },
            |block| {
                populate_false_block(block);
                unused_variable
            },
        );
    }

    pub(super) fn if_else_with_result(
        &mut self,
        condition: Variable1,
        populate_true_block: impl Fn(&mut CompilerVisitor) -> Variable8,
        populate_false_block: impl Fn(&mut CompilerVisitor) -> Variable8,
    ) -> Variable8 {
        let r#true = self.define_1(true);

        let variable_id_counter = Rc::clone(&self.current_block.borrow().variable_id_counter);

        let exit_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        exit_block.borrow_mut().set_has_argument(true);

        let true_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        let mut true_block_visitor = CompilerVisitor {
            current_block: true_block,
        };
        let true_value = populate_true_block(&mut true_block_visitor);
        let true_block = true_block_visitor.current_block;
        true_block.borrow_mut().jump = Jump::BasicBlock {
            condition: r#true,
            target_if_true: Rc::clone(&exit_block),
            target_if_true_argument: Some(true_value),
            target_if_false: Rc::clone(&exit_block),
            target_if_false_argument: Some(true_value),
        };

        let false_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        let mut false_block_visitor = CompilerVisitor {
            current_block: false_block,
        };
        let false_value = populate_false_block(&mut false_block_visitor);
        let false_block = false_block_visitor.current_block;
        false_block.borrow_mut().jump = Jump::BasicBlock {
            condition: r#true,
            target_if_true: Rc::clone(&exit_block),
            target_if_true_argument: Some(false_value),
            target_if_false: Rc::clone(&exit_block),
            target_if_false_argument: Some(false_value),
        };

        self.current_block.borrow_mut().jump = Jump::BasicBlock {
            condition,
            target_if_true: Rc::clone(&true_block),
            target_if_true_argument: None,
            target_if_false: Rc::clone(&false_block),
            target_if_false_argument: None,
        };

        let result = exit_block
            .borrow_mut()
            .define_8(Definition8::BasicBlockArgument);
        self.current_block = exit_block;
        result
    }

    pub(crate) fn jump(&self, jump: Jump) {
        self.current_block.borrow_mut().jump = jump;
    }
}
