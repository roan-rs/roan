pub mod native_fn;

use crate::value::Value;
use roan_error::frame::Frame;

/// Virtual machine for executing Roan code.
#[derive(Debug, Clone)]
pub struct VM {
    /// The stack of frames.
    frames: Vec<Frame>,
    /// The stack of values.
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            frames: vec![],
            stack: vec![],
        }
    }
}

impl VM {
    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop()
    }

    pub fn frame(&self) -> Option<&Frame> {
        self.frames.last()
    }

    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }
}

impl VM {
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<&Value> {
        self.stack.last()
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    pub fn stack_last(&self) -> Option<&Value> {
        self.stack.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use roan_error::TextSpan;

    #[test]
    fn test_vm() {
        let mut vm = VM::new();
        assert_eq!(vm.frames().len(), 0);
        assert_eq!(vm.stack().len(), 0);

        let frame = Frame::new(
            "test".to_string(),
            TextSpan::default(),
            ".\\test.roan".to_string(),
        );
        vm.push_frame(frame.clone());
        assert_eq!(vm.frames().len(), 1);

        let value = Value::Int(42);
        vm.push(value.clone());
        assert_eq!(vm.stack().len(), 1);
        assert_eq!(vm.peek(), Some(&value));

        let popped = vm.pop().unwrap();
        assert_eq!(popped, value);
        assert_eq!(vm.stack().len(), 0);

        let popped = vm.pop_frame().unwrap();
        assert_eq!(vm.frames().len(), 0);
    }
}
