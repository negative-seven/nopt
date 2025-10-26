mod r#abstract;
mod instruction_decoder;

use crate::{
    compiler::{
        frontend::r#abstract::{Compiler, Visitor},
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
    .compile();

    (Function { basic_block }, is_prg_rom_only)
}

pub(crate) struct CompilerVisitor {
    current_block: Rc<RefCell<BasicBlock>>,
}

impl CompilerVisitor {
    fn define_1(&mut self, definition: Definition1) -> Variable1 {
        self.current_block.borrow_mut().define_1(definition)
    }

    fn define_8(&mut self, definition: Definition8) -> Variable8 {
        self.current_block.borrow_mut().define_8(definition)
    }

    fn define_16(&mut self, definition: Definition16) -> Variable16 {
        self.current_block.borrow_mut().define_16(definition)
    }

    fn store_1(&mut self, destination: Destination1, value: Variable1) {
        self.current_block
            .borrow_mut()
            .instructions
            .push(Instruction::Store1 {
                destination,
                variable: value,
            });
    }

    fn store_8(&mut self, destination: Destination8, value: Variable8) {
        self.current_block
            .borrow_mut()
            .instructions
            .push(Instruction::Store8 {
                destination,
                variable: value,
            });
    }

    fn store_16(&mut self, destination: Destination16, value: Variable16) {
        self.current_block
            .borrow_mut()
            .instructions
            .push(Instruction::Store16 {
                destination,
                variable: value,
            });
    }
}

impl Visitor for CompilerVisitor {
    type U1 = Variable1;
    type U8 = Variable8;
    type U16 = Variable16;

    fn immediate_u8(&mut self, value: u8) -> Variable8 {
        self.define_8(Definition8::Immediate(value))
    }

    fn cpu_c(&mut self) -> Variable1 {
        self.define_1(Definition1::CpuFlag(CpuFlag::C))
    }

    fn set_cpu_c(&mut self, value: Variable1) {
        self.store_1(Destination1::CpuFlag(CpuFlag::C), value);
    }

    fn cpu_z(&mut self) -> Variable1 {
        self.define_1(Definition1::CpuFlag(CpuFlag::Z))
    }

    fn set_cpu_z(&mut self, value: Variable1) {
        self.store_1(Destination1::CpuFlag(CpuFlag::Z), value);
    }

    fn set_cpu_i(&mut self, value: Variable1) {
        self.store_1(Destination1::CpuFlag(CpuFlag::I), value);
    }

    fn set_cpu_d(&mut self, value: Variable1) {
        self.store_1(Destination1::CpuFlag(CpuFlag::D), value);
    }

    fn cpu_b(&mut self) -> Variable1 {
        self.define_1(Definition1::CpuFlag(CpuFlag::B))
    }

    fn set_cpu_b(&mut self, value: Variable1) {
        self.store_1(Destination1::CpuFlag(CpuFlag::B), value);
    }

    fn cpu_unused_flag(&mut self) -> Variable1 {
        self.define_1(Definition1::CpuFlag(CpuFlag::Unused))
    }

    fn set_cpu_unused_flag(&mut self, value: Variable1) {
        self.store_1(Destination1::CpuFlag(CpuFlag::Unused), value);
    }

    fn cpu_v(&mut self) -> Variable1 {
        self.define_1(Definition1::CpuFlag(CpuFlag::V))
    }

    fn set_cpu_v(&mut self, value: Variable1) {
        self.store_1(Destination1::CpuFlag(CpuFlag::V), value);
    }

    fn cpu_n(&mut self) -> Variable1 {
        self.define_1(Definition1::CpuFlag(CpuFlag::N))
    }

    fn set_cpu_n(&mut self, value: Variable1) {
        self.store_1(Destination1::CpuFlag(CpuFlag::N), value);
    }

    fn cpu_a(&mut self) -> Variable8 {
        self.define_8(Definition8::CpuRegister(CpuRegister::A))
    }

    fn set_cpu_a(&mut self, value: Variable8) {
        self.store_8(Destination8::CpuRegister(CpuRegister::A), value);
    }

    fn cpu_x(&mut self) -> Variable8 {
        self.define_8(Definition8::CpuRegister(CpuRegister::X))
    }

    fn set_cpu_x(&mut self, value: Variable8) {
        self.store_8(Destination8::CpuRegister(CpuRegister::X), value);
    }

    fn cpu_y(&mut self) -> Variable8 {
        self.define_8(Definition8::CpuRegister(CpuRegister::Y))
    }

    fn set_cpu_y(&mut self, value: Variable8) {
        self.store_8(Destination8::CpuRegister(CpuRegister::Y), value);
    }

    fn cpu_s(&mut self) -> Variable8 {
        self.define_8(Definition8::CpuRegister(CpuRegister::S))
    }

    fn set_cpu_s(&mut self, value: Variable8) {
        self.store_8(Destination8::CpuRegister(CpuRegister::S), value);
    }

    fn cpu_p(&mut self) -> Variable8 {
        self.define_8(Definition8::CpuRegister(CpuRegister::P))
    }

    fn set_cpu_p(&mut self, value: Variable8) {
        self.store_8(Destination8::CpuRegister(CpuRegister::P), value);
    }

    fn cpu_pc(&mut self) -> Variable16 {
        self.define_16(Definition16::Pc)
    }

    fn set_cpu_pc(&mut self, value: Variable16) {
        self.store_16(Destination16::CpuPc, value);
    }

    fn ppu_current_address(&mut self) -> Variable16 {
        self.define_16(Definition16::PpuCurrentAddress)
    }

    fn set_ppu_current_address(&mut self, value: Variable16) {
        self.store_16(Destination16::PpuCurrentAddress, value);
    }

    fn cpu_ram(&mut self, address: Variable16) -> Variable8 {
        self.define_8(Definition8::CpuRam(address))
    }

    fn set_cpu_ram(&mut self, address: Variable16, value: Variable8) {
        self.store_8(Destination8::CpuRam(address), value);
    }

    fn prg_ram(&mut self, address: Variable16) -> Variable8 {
        self.define_8(Definition8::PrgRam(address))
    }

    fn set_prg_ram(&mut self, address: Variable16, value: Variable8) {
        self.store_8(Destination8::PrgRam(address), value);
    }

    fn ppu_ram(&mut self, address: Variable16) -> Variable8 {
        self.define_8(Definition8::PpuRam(address))
    }

    fn set_ppu_ram(&mut self, address: Variable16, value: Variable8) {
        self.store_8(Destination8::PpuRam(address), value);
    }

    fn rom(&mut self, address: Variable16) -> Variable8 {
        self.define_8(Definition8::Rom(address))
    }

    fn get_bit(&mut self, value: Variable8, bit_index: u8) -> Variable1 {
        self.define_1(Definition1::U8Bit {
            operand: value,
            index: bit_index,
        })
    }

    fn not(&mut self, operand: Variable1) -> Variable1 {
        self.define_1(Definition1::Not(operand))
    }

    fn is_zero(&mut self, operand: Variable8) -> Variable1 {
        self.define_1(Definition1::EqualToZero(operand))
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
        value_if_true: Variable16,
        value_if_false: Variable16,
    ) -> Variable16 {
        self.define_16(Definition16::Select {
            condition,
            result_if_true: value_if_true,
            result_if_false: value_if_false,
        })
    }

    fn concatenate(&mut self, operand_0: Variable8, operand_1: Variable8) -> Variable16 {
        self.define_16(Definition16::FromU8s {
            high: operand_0,
            low: operand_1,
        })
    }

    fn or(&mut self, operand_0: Variable8, operand_1: Variable8) -> Variable8 {
        self.define_8(Definition8::Or(operand_0, operand_1))
    }

    fn and_u1(&mut self, operand_0: Variable1, operand_1: Variable1) -> Variable1 {
        self.define_1(Definition1::And(operand_0, operand_1))
    }

    fn and_u8(&mut self, operand_0: Variable8, operand_1: Variable8) -> Variable8 {
        self.define_8(Definition8::And(operand_0, operand_1))
    }

    fn xor(&mut self, operand_0: Variable8, operand_1: Variable8) -> Variable8 {
        self.define_8(Definition8::Xor(operand_0, operand_1))
    }

    fn add_with_carry_u8(
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

    fn add_with_carry_u8_carry(
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

    fn add_with_carry_u8_overflow(
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

    fn sub_with_borrow(
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

    fn sub_with_borrow_borrow(
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

    fn sub_with_borrow_overflow(
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

    fn if_else(
        &mut self,
        condition: Variable1,
        visit_true: impl Fn(CompilerVisitor),
        visit_false: impl Fn(CompilerVisitor),
    ) {
        let unused_variable = self.define_8(Definition8::Immediate(0));
        self.if_else_with_result(
            condition,
            |block| {
                visit_true(block);
                unused_variable
            },
            |block| {
                visit_false(block);
                unused_variable
            },
        );
    }

    fn if_else_with_result(
        &mut self,
        condition: Variable1,
        visit_true: impl Fn(CompilerVisitor) -> Variable8,
        visit_false: impl Fn(CompilerVisitor) -> Variable8,
    ) -> Variable8 {
        let r#true = self.immediate_u1(true);

        let variable_id_counter = Rc::clone(&self.current_block.borrow().variable_id_counter);

        let exit_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        exit_block.borrow_mut().set_has_argument(true);

        let true_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        let true_block_visitor = CompilerVisitor {
            current_block: Rc::clone(&true_block),
        };
        let true_value = visit_true(true_block_visitor);
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
        let false_block_visitor = CompilerVisitor {
            current_block: Rc::clone(&false_block),
        };
        let false_value = visit_false(false_block_visitor);
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

    fn terminate(self) {
        self.current_block.borrow_mut().jump = Jump::Return;
    }
}
