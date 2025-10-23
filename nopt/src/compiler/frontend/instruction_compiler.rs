use std::{cell::RefCell, rc::Rc, sync::atomic::AtomicUsize};
use crate::{
    compiler::{
        frontend::abstract_instruction_compiler::AbstractInstructionCompiler,
        ir::{
            BasicBlock, Definition1, Definition8, Definition16, Destination1, Destination8,
            Function, Instruction, Jump, Variable1, Variable8, Variable16,
        },
    },
    nes_assembly,
};

pub(super) fn compile(instruction: nes_assembly::Instruction) -> Function {
    let basic_block = Rc::new(RefCell::new(BasicBlock::new(Rc::new(AtomicUsize::new(0)))));
    AbstractInstructionCompiler {
        visitor: CompilerVisitor {
            current_block: Rc::clone(&basic_block),
        },
        cpu_instruction: instruction,
    }
    .transpile();
    Function { basic_block }
}

pub(crate) struct CompilerVisitor {
    current_block: Rc<RefCell<BasicBlock>>,
}

impl CompilerVisitor {
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

    pub(super) fn if_else(
        &mut self,
        condition: Variable1,
        populate_true_block: impl Fn(&mut BasicBlock),
        populate_false_block: impl Fn(&mut BasicBlock),
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
        populate_true_block: impl Fn(&mut BasicBlock) -> Variable8,
        populate_false_block: impl Fn(&mut BasicBlock) -> Variable8,
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
        let true_value = populate_true_block(&mut true_block.borrow_mut());
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
        let false_value = populate_false_block(&mut false_block.borrow_mut());
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
