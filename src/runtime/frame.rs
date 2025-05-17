use super::object::Object;
use runevm_classfile::{Constant, ConstantPool, Instruction, MethodInfo};

macro_rules! unwrap_constant {
    ($cp:expr, $method:ident, $idx:expr) => {{
        let constant = $cp.$method($idx);
        ($cp.class(constant.0), $cp.name_and_type(constant.1))
    }};
}

pub struct Frame {
    constant_pool: ConstantPool,
    method: MethodInfo,
    pc: usize,
    operand_stack: Vec<OperandItem>,
}

impl Frame {
    pub fn new(constant_pool: &ConstantPool, method: MethodInfo) -> Frame {
        Frame {
            constant_pool: constant_pool.clone(),
            method,
            pc: 0,
            operand_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self) -> Result<FrameResult, FrameError> {
        let code = self.method.code();

        while self.pc < code.len() {
            let inst = code[self.pc];
            print!("{:?} ", code[self.pc]);

            match inst {
                Instruction::Getstatic(index) => {
                    let (class, name_and_type) = unwrap_constant!(self.constant_pool, field, index);
                    print!("{} {} {}", class, name_and_type.0, name_and_type.1);
                }
                Instruction::Ldc(index) => match self.constant_pool.get(index as u16) {
                    Constant::String(string_index) => {
                        print!("\"{}\"", self.constant_pool.utf8(*string_index));
                    },
                    Constant::Integer(value) => {
                        self.operand_stack.push(OperandItem::Integer(*value));
                    },
                    _ => todo!(),
                },
                Instruction::Bipush(value) => {
                    self.operand_stack.push(OperandItem::Integer(value as i32));

                },
                Instruction::Istore(index) => {
                    let value = self.operand_stack.pop().ok_or(FrameError::StackUnderflow)?;
                    if let OperandItem::Integer(value) = value {
                        print!("{} = {}", index, value);
                        // TODO: Store value in local variable
                    } else {
                        panic!("Expected integer, got {value:?}")
                    }
                },
                Instruction::Invokevirtual(index) => {
                    let (class, name_and_type) = unwrap_constant!(self.constant_pool, method, index);
                    print!("{} {} {}", class, name_and_type.0, name_and_type.1);
                },
                _ => {}
            }

            println!();
            self.pc += 1;
        }

        Ok(FrameResult::Finished)
    }
}

pub enum FrameResult {
    NextFrame(MethodInfo),
    Finished,
}

#[derive(Debug)]
pub enum OperandItem {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Reference(Object),
    Padding,
}

#[derive(Debug)]
pub enum FrameError {
    StackUnderflow,
}