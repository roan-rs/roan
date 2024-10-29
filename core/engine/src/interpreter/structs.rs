use std::collections::HashMap;
use crate::{
    context::Context,
    module::{ExportType, Module, StoredImpl, StoredStruct, StoredTraitImpl},
};
use anyhow::Result;
use log::debug;
use roan_ast::{Struct, StructConstructor, StructImpl, TraitDef, TraitImpl};
use roan_error::{error::PulseError, TextSpan};
use crate::value::Value;
use crate::vm::VM;

impl Module {
    /// Interpret a struct implementation.
    ///
    /// # Arguments
    /// * `impl_stmt` - [`StructImpl`] - The struct implementation to interpret.
    pub fn interpret_struct_impl(
        &mut self,
        impl_stmt: StructImpl,
        ctx: &mut Context,
    ) -> Result<()> {
        let struct_name = impl_stmt.struct_name.literal();

        let mut found_struct = self.get_struct(&struct_name, impl_stmt.struct_name.span.clone())?;
        let stored_impl = StoredImpl {
            def: impl_stmt.clone(),
            defining_module: self.id(),
        };
        found_struct.impls.push(stored_impl.clone());

        if let Some(existing_struct) = self
            .structs
            .iter_mut()
            .find(|s| s.name.literal() == struct_name)
        {
            *existing_struct = found_struct;

            if let Some(export) = self.exports.iter_mut().find(|(n, _)| n == &struct_name) {
                if let ExportType::Struct(s) = &mut export.1 {
                    s.impls.push(stored_impl);
                }
            }
        }

        ctx.upsert_module(self.id().clone(), self.clone());

        Ok(())
    }

    /// Interpret a trait implementation.
    ///
    /// # Arguments
    /// * `impl_stmt` - [`TraitImpl`] - The trait implementation to interpret.
    pub fn interpret_trait_impl(&mut self, impl_stmt: TraitImpl, ctx: &mut Context) -> Result<()> {
        let for_name = impl_stmt.struct_name.literal();
        let trait_name = impl_stmt.trait_name.literal();

        let mut struct_def = self.get_struct(&for_name, impl_stmt.struct_name.span.clone())?;
        let trait_def = self.get_trait(&trait_name, impl_stmt.trait_name.span.clone())?;
        if struct_def
            .trait_impls
            .iter()
            .any(|t| t.def.trait_name.literal() == trait_name)
        {
            return Err(PulseError::StructAlreadyImplementsTrait(
                for_name,
                trait_name,
                impl_stmt.trait_name.span.clone(),
            )
            .into());
        }

        let missing_methods: Vec<String> = trait_def
            .methods
            .iter()
            .filter(|m| !impl_stmt.methods.iter().any(|i| i.name == m.name))
            .map(|m| m.name.clone())
            .collect();

        if !missing_methods.is_empty() {
            return Err(PulseError::TraitMethodNotImplemented(
                trait_name,
                missing_methods,
                impl_stmt.trait_name.span.clone(),
            )
            .into());
        }

        let stored_trait_impl = StoredTraitImpl {
            def: impl_stmt.clone(),
            defining_module: self.id(),
        };

        struct_def.trait_impls.push(stored_trait_impl.clone());

        if let Some(existing_struct) = self
            .structs
            .iter_mut()
            .find(|s| s.name.literal() == for_name)
        {
            *existing_struct = struct_def;

            if let Some(export) = self.exports.iter_mut().find(|(n, _)| n == &for_name) {
                if let ExportType::Struct(s) = &mut export.1 {
                    s.trait_impls.push(stored_trait_impl);
                }
            }
        }

        ctx.upsert_module(self.id().clone(), self.clone());

        Ok(())
    }

    pub fn get_trait(&self, name: &str, span: TextSpan) -> Result<TraitDef> {
        Ok(self
            .traits
            .iter()
            .find(|t| t.name.literal() == name)
            .cloned()
            .ok_or_else(|| PulseError::TraitNotFoundError(name.into(), span))?)
    }

    pub fn get_struct(&self, name: &str, span: TextSpan) -> Result<StoredStruct> {
        let x = self.structs.iter().find(|s| s.name.literal() == name);

        Ok(x.cloned()
            .ok_or_else(|| PulseError::StructNotFoundError(name.into(), span))?)
    }

    /// Interpret a struct definition.
    ///
    /// # Arguments
    /// * `struct_def` - [`Struct`] - The struct definition to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the struct definition.
    ///
    /// # Returns
    /// The result of interpreting the struct definition.
    pub fn interpret_struct(&mut self, struct_stmt: Struct, ctx: &mut Context) -> Result<()> {
        let def = struct_stmt.clone();
        let stored_struct = StoredStruct {
            defining_module: self.id(),
            struct_token: def.struct_token,
            name: def.name,
            fields: def.fields,
            public: def.public,
            impls: vec![],
            trait_impls: vec![],
        };

        self.structs.push(stored_struct.clone());

        if struct_stmt.public {
            self.exports.push((
                struct_stmt.name.literal(),
                ExportType::Struct(stored_struct),
            ));
        }

        ctx.upsert_module(self.id().clone(), self.clone());
    
        Ok(())
    }
    
    /// Interpret trait definition.
    /// 
    /// # Arguments
    /// * `trait_def` - [`TraitDef`] - The trait definition to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the trait definition.
    /// 
    /// # Returns
    /// The result of interpreting the trait definition.
    pub fn interpret_trait(&mut self, trait_stmt: TraitDef, ctx: &mut Context) -> Result<()> {
        self.traits.push(trait_stmt.clone());

        if trait_stmt.public {
            self.exports.push((
                trait_stmt.name.literal(),
                ExportType::Trait(trait_stmt.clone()),
            ));
        }

        ctx.upsert_module(self.id().clone(), self.clone());
        
        Ok(())
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
