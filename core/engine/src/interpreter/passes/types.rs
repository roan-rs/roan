use crate::{context::Context, interpreter::passes::Pass, module::Module, vm::VM};
use roan_ast::Stmt;

#[derive(Clone)]
pub struct TypePass;

impl Pass for TypePass {
    fn pass_stmt(
        &mut self,
        stmt: Stmt,
        module: &mut Module,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> anyhow::Result<()> {
        match stmt {
            Stmt::Fn(func) => {
                println!("Function: {:?}", func.params);
            }
            _ => {}
        }
        
        Ok(())
    }
}
