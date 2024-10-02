#[derive(Debug, Clone)]
pub struct CallFrame {
    pub module: Module,
    pub ip: usize,
    pub stack: Vec<Value>,
}