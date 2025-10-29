mod instruction_decoder;
pub(crate) mod nes;

use crate::compiler::{
    frontend::nes::{Cpu, Nes, Visitor},
    ir::{
        BasicBlock, Definition1, Definition8, Definition16, Destination8, Function, Instruction,
        Jump, Variable1, Variable8, Variable16,
    },
};
use std::{cell::RefCell, rc::Rc, sync::atomic::AtomicUsize};

pub(super) fn compile_instruction<Cartridge: crate::cartridge::Cartridge>(
    nes: &mut Nes<Cartridge>,
    address: u16,
) -> (Function, bool) {
    let (cpu_instruction, is_prg_rom_only) = instruction_decoder::decode_instruction(nes, address);

    let basic_block = Rc::new(RefCell::new(BasicBlock::new(Rc::new(AtomicUsize::new(0)))));
    Cpu::compile(
        nes,
        CompilerVisitor {
            current_block: Rc::clone(&basic_block),
            exit_block: None,
        },
        &cpu_instruction,
    );

    (Function { basic_block }, is_prg_rom_only)
}

pub(crate) struct CompilerVisitor {
    current_block: Rc<RefCell<BasicBlock>>,
    exit_block: Option<Rc<RefCell<BasicBlock>>>,
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

    fn store_8(&mut self, destination: Destination8, value: Variable8) {
        self.current_block
            .borrow_mut()
            .instructions
            .push(Instruction::Store8 {
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

    fn memory_with_offset_u8(&mut self, address: *const u8, offset: Variable16) -> Variable8 {
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_memory_with_offset_u8(
        &mut self,
        address: *mut u8,
        offset: Variable16,
        value: Variable8,
    ) {
        self.store_8(Destination8::NativeMemory { address, offset }, value);
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
        mut visit_true: impl FnMut(CompilerVisitor),
        mut visit_false: impl FnMut(CompilerVisitor),
    ) {
        let variable_id_counter = Rc::clone(&self.current_block.borrow().variable_id_counter);

        let exit_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));

        let true_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        visit_true(CompilerVisitor {
            current_block: Rc::clone(&true_block),
            exit_block: Some(Rc::clone(&exit_block)),
        });

        let false_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        visit_false(CompilerVisitor {
            current_block: Rc::clone(&false_block),
            exit_block: Some(Rc::clone(&exit_block)),
        });

        self.current_block.borrow_mut().jump = Jump::BasicBlock {
            condition,
            target_if_true: Rc::clone(&true_block),
            target_if_true_argument: None,
            target_if_false: Rc::clone(&false_block),
            target_if_false_argument: None,
        };

        self.current_block = exit_block;
    }

    fn if_else_with_result(
        &mut self,
        condition: Variable1,
        mut visit_true: impl FnMut(CompilerVisitor),
        mut visit_false: impl FnMut(CompilerVisitor),
    ) -> Variable8 {
        let variable_id_counter = Rc::clone(&self.current_block.borrow().variable_id_counter);

        let exit_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        exit_block.borrow_mut().set_has_argument(true);

        let true_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        visit_true(CompilerVisitor {
            current_block: Rc::clone(&true_block),
            exit_block: Some(Rc::clone(&exit_block)),
        });

        let false_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        visit_false(CompilerVisitor {
            current_block: Rc::clone(&false_block),
            exit_block: Some(Rc::clone(&exit_block)),
        });

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

    fn terminate(mut self, argument: Option<Variable8>) {
        let r#true = self.immediate_u1(true);
        self.current_block.borrow_mut().jump = if let Some(exit_block) = self.exit_block {
            Jump::BasicBlock {
                condition: r#true,
                target_if_true: Rc::clone(&exit_block),
                target_if_true_argument: argument,
                target_if_false: exit_block,
                target_if_false_argument: argument,
            }
        } else {
            assert!(argument.is_none());
            Jump::Return
        };
    }
}
