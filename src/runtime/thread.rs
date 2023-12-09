use super::frame::{Frame, FrameResult};
use runevm_classfile::{ConstantPool, MethodInfo};

pub struct JavaThread {
    stack: Vec<Frame>,
}

impl JavaThread {
    pub fn new(constant_pool: &ConstantPool, method: MethodInfo) -> JavaThread {
        let mut stack: Vec<Frame> = Vec::new();
        stack.push(Frame::new(constant_pool, method));

        JavaThread { stack }
    }

    pub fn run(&mut self) {
        while !self.stack.is_empty() {
            let mut current = self.stack.pop().unwrap();

            match current.execute() {
                FrameResult::NextFrame(_) => todo!(),
                FrameResult::Finished => {}
            }
        }
    }
}
