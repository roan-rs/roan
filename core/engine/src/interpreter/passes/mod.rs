pub mod imports;
pub mod resolver;
pub mod types;

use crate::{context::Context, module::Module, vm::VM};
use dyn_clone::{clone_trait_object, DynClone};
use roan_ast::Stmt;

pub trait Pass: DynClone {
    fn run(&mut self, module: &mut Module, ctx: &mut Context, vm: &mut VM) -> anyhow::Result<()> {
        for stmt in module.ast.stmts.clone() {
            self.pass_stmt(stmt, module, ctx, vm)?;
        }

        Ok(())
    }

    fn pass_stmt(
        &mut self,
        stmt: Stmt,
        module: &mut Module,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> anyhow::Result<()>;
}

clone_trait_object!(Pass);
