use crate::{
    context::Context,
    module::{ExportType, Module, StoredImpl, StoredStruct, StoredTraitImpl},
    value::Value,
    vm::VM,
};
use anyhow::Result;
use log::debug;
use roan_ast::{Struct, StructConstructor, StructImpl, TraitDef, TraitImpl};
use roan_error::{error::RoanError, TextSpan};
use std::collections::HashMap;

impl Module {
    pub fn get_trait(&self, name: &str, span: TextSpan) -> Result<TraitDef> {
        Ok(self
            .traits
            .iter()
            .find(|t| t.name.literal() == name)
            .cloned()
            .ok_or_else(|| RoanError::TraitNotFoundError(name.into(), span))?)
    }

    pub fn get_struct(&self, name: &str, span: TextSpan) -> Result<StoredStruct> {
        let x = self.structs.iter().find(|s| s.name.literal() == name);

        Ok(x.cloned()
            .ok_or_else(|| RoanError::StructNotFoundError(name.into(), span))?)
    }

    /// Interpret a struct constructor expression.
    ///
    /// # Arguments
    /// * `constructor` - [StructConstructor] expression to interpret.
    /// * `ctx` - The context in which to interpret the struct constructor expression.
    /// * `vm` - The virtual machine to use.
    ///
    /// # Returns
    /// The result of the struct constructor expression.
    pub fn interpret_struct_constructor(
        &mut self,
        constructor: StructConstructor,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<Value> {
        debug!("Interpreting struct constructor");
        let found = self.get_struct(&constructor.name, constructor.token.span.clone())?;

        let mut fields = HashMap::new();

        for (field_name, expr) in constructor.fields.iter() {
            self.interpret_expr(expr, ctx, vm)?;
            fields.insert(field_name.clone(), vm.pop().unwrap());
        }

        Ok(Value::Struct(found, fields))
    }
}
