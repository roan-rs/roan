pub mod native_fn;

use roan_error::frame::Frame;
use crate::value::Value;

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
}
