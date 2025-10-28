mod r#abstract;
mod instruction_decoder;

use crate::{
    compiler::{
        frontend::r#abstract::{Compiler, Visitor},
        ir::{
            BasicBlock, Definition1, Definition8, Definition16, Destination8, Destination16,
            Function, Instruction, Jump, Variable1, Variable8, Variable16,
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
            nes,
            current_block: Rc::clone(&basic_block),
            exit_block: None,
        },
        cpu_instruction: nes_instruction,
    }
    .compile();

    (Function { basic_block }, is_prg_rom_only)
}

pub(crate) struct CompilerVisitor {
    nes: *mut Nes,
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

    fn store_16(&mut self, destination: Destination16, value: Variable16) {
        self.current_block
            .borrow_mut()
            .instructions
            .push(Instruction::Store16 {
                destination,
                variable: value,
            });
    }

    fn get_cpu_flag<const INDEX: u8>(&mut self) -> Variable1 {
        let p = self.cpu_p();
        self.get_bit(p, INDEX)
    }

    fn set_cpu_flag<const INDEX: u8>(&mut self, value: Variable1) {
        let clear_bit_mask = self.immediate_u8(!(1 << INDEX));

        let p = self.cpu_p();
        let p = self.and_u8(p, clear_bit_mask);
        let p = self.if_else_with_result(
            value,
            |mut visitor| {
                let set_bit_mask = visitor.immediate_u8(1 << INDEX);
                let p = visitor.or(p, set_bit_mask);
                visitor.terminate(Some(p));
            },
            |visitor| {
                visitor.terminate(Some(p));
            },
        );
        self.set_cpu_p(p);
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
        self.get_cpu_flag::<0>()
    }

    fn set_cpu_c(&mut self, value: Variable1) {
        self.set_cpu_flag::<0>(value);
    }

    fn cpu_z(&mut self) -> Variable1 {
        self.get_cpu_flag::<1>()
    }

    fn set_cpu_z(&mut self, value: Variable1) {
        self.set_cpu_flag::<1>(value);
    }

    fn set_cpu_i(&mut self, value: Variable1) {
        self.set_cpu_flag::<2>(value);
    }

    fn set_cpu_d(&mut self, value: Variable1) {
        self.set_cpu_flag::<3>(value);
    }

    fn cpu_b(&mut self) -> Variable1 {
        self.get_cpu_flag::<4>()
    }

    fn set_cpu_b(&mut self, value: Variable1) {
        self.set_cpu_flag::<4>(value);
    }

    fn cpu_unused_flag(&mut self) -> Variable1 {
        self.get_cpu_flag::<5>()
    }

    fn set_cpu_unused_flag(&mut self, value: Variable1) {
        self.set_cpu_flag::<5>(value);
    }

    fn cpu_v(&mut self) -> Variable1 {
        self.get_cpu_flag::<6>()
    }

    fn set_cpu_v(&mut self, value: Variable1) {
        self.set_cpu_flag::<6>(value);
    }

    fn cpu_n(&mut self) -> Variable1 {
        self.get_cpu_flag::<7>()
    }

    fn set_cpu_n(&mut self, value: Variable1) {
        self.set_cpu_flag::<7>(value);
    }

    fn cpu_a(&mut self) -> Variable8 {
        let address = unsafe { &raw const (*self.nes).cpu.a };
        let offset = self.immediate_u16(0);
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_cpu_a(&mut self, value: Variable8) {
        let address = unsafe { &raw mut (*self.nes).cpu.a };
        let offset = self.immediate_u16(0);
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn cpu_x(&mut self) -> Variable8 {
        let address = unsafe { &raw const (*self.nes).cpu.x };
        let offset = self.immediate_u16(0);
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_cpu_x(&mut self, value: Variable8) {
        let address = unsafe { &raw mut (*self.nes).cpu.x };
        let offset = self.immediate_u16(0);
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn cpu_y(&mut self) -> Variable8 {
        let address = unsafe { &raw const (*self.nes).cpu.y };
        let offset = self.immediate_u16(0);
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_cpu_y(&mut self, value: Variable8) {
        let address = unsafe { &raw mut (*self.nes).cpu.y };
        let offset = self.immediate_u16(0);
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn cpu_s(&mut self) -> Variable8 {
        let address = unsafe { &raw const (*self.nes).cpu.s };
        let offset = self.immediate_u16(0);
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_cpu_s(&mut self, value: Variable8) {
        let address = unsafe { &raw mut (*self.nes).cpu.s };
        let offset = self.immediate_u16(0);
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn cpu_p(&mut self) -> Variable8 {
        let address = unsafe { &raw const (*self.nes).cpu.p };
        let offset = self.immediate_u16(0);
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_cpu_p(&mut self, value: Variable8) {
        let address = unsafe { &raw mut (*self.nes).cpu.p };
        let offset = self.immediate_u16(0);
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn cpu_pc(&mut self) -> Variable16 {
        let address = unsafe { &raw const (*self.nes).cpu.pc };
        self.define_16(Definition16::NativeMemory { address })
    }

    fn set_cpu_pc(&mut self, value: Variable16) {
        let address = unsafe { &raw mut (*self.nes).cpu.pc };
        self.store_16(Destination16::NativeMemory { address }, value);
    }

    fn ppu_control_register(&mut self) -> Variable8 {
        let address = unsafe { &raw const (*self.nes).ppu.control_register };
        let offset = self.immediate_u16(0);
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_ppu_control_register(&mut self, value: Variable8) {
        let address = unsafe { &raw mut (*self.nes).ppu.control_register };
        let offset = self.immediate_u16(0);
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn ppu_read_buffer(&mut self) -> Variable8 {
        let address = unsafe { &raw const (*self.nes).ppu.read_buffer };
        let offset = self.immediate_u16(0);
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_ppu_read_buffer(&mut self, value: Variable8) {
        let address = unsafe { &raw mut (*self.nes).ppu.read_buffer };
        let offset = self.immediate_u16(0);
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn ppu_current_address(&mut self) -> Variable16 {
        let address = unsafe { &raw const (*self.nes).ppu.current_address };
        self.define_16(Definition16::NativeMemory { address })
    }

    fn set_ppu_current_address(&mut self, value: Variable16) {
        let address = unsafe { &raw mut (*self.nes).ppu.current_address };
        self.store_16(Destination16::NativeMemory { address }, value);
    }

    fn cpu_ram(&mut self, offset: Variable16) -> Variable8 {
        let address = unsafe { (*self.nes).cpu.ram.as_ptr() };
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_cpu_ram(&mut self, offset: Variable16, value: Variable8) {
        let address = unsafe { (*self.nes).cpu.ram.as_mut_ptr() };
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn prg_ram(&mut self, offset: Variable16) -> Variable8 {
        let address = unsafe { (*self.nes).prg_ram.as_ptr() };
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_prg_ram(&mut self, offset: Variable16, value: Variable8) {
        let address = unsafe { (*self.nes).prg_ram.as_mut_ptr() };
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn ppu_ram(&mut self, offset: Variable16) -> Variable8 {
        let address = unsafe { (*self.nes).ppu.ram.as_ptr() };
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_ppu_ram(&mut self, offset: Variable16, value: Variable8) {
        let address = unsafe { (*self.nes).ppu.ram.as_mut_ptr() };
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn ppu_palette_ram(&mut self, offset: Variable16) -> Variable8 {
        let address = unsafe { (*self.nes).ppu.palette_ram.as_ptr() };
        self.define_8(Definition8::NativeMemory { address, offset })
    }

    fn set_ppu_palette_ram(&mut self, offset: Variable16, value: Variable8) {
        let address = unsafe { (*self.nes).ppu.palette_ram.as_mut_ptr() };
        self.store_8(Destination8::NativeMemory { address, offset }, value);
    }

    fn rom(&mut self, offset: Variable16) -> Variable8 {
        let address = unsafe { (*self.nes).rom.prg_rom().as_ptr() };
        self.define_8(Definition8::NativeMemory { address, offset })
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
        let variable_id_counter = Rc::clone(&self.current_block.borrow().variable_id_counter);

        let exit_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));

        let true_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        visit_true(CompilerVisitor {
            nes: self.nes,
            current_block: Rc::clone(&true_block),
            exit_block: Some(Rc::clone(&exit_block)),
        });

        let false_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        visit_false(CompilerVisitor {
            nes: self.nes,
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
        visit_true: impl Fn(CompilerVisitor),
        visit_false: impl Fn(CompilerVisitor),
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
            nes: self.nes,
            current_block: Rc::clone(&true_block),
            exit_block: Some(Rc::clone(&exit_block)),
        });

        let false_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &variable_id_counter,
        ))));
        visit_false(CompilerVisitor {
            nes: self.nes,
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
