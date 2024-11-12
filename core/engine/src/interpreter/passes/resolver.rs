use crate::{
    context::Context,
    interpreter::passes::Pass,
    module::{
        ExportType, Module, StoredConst, StoredFunction, StoredImpl, StoredStruct, StoredTraitImpl,
    },
    vm::VM,
};
use anyhow::Result;
use roan_ast::{Const, Stmt, Struct, StructImpl, TraitDef, TraitImpl};
use roan_error::{error::RoanError, TextSpan};
use tracing::debug;

#[derive(Clone)]
pub struct ResolverPass;

impl Pass for ResolverPass {
    fn pass_stmt(
        &mut self,
        stmt: Stmt,
        module: &mut Module,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<()> {
        match stmt {
            Stmt::Fn(f) => self.interpret_function(module, f, ctx)?,
            Stmt::Struct(struct_stmt) => self.interpret_struct(module, struct_stmt, ctx)?,
            Stmt::TraitDef(trait_stmt) => self.interpret_trait(module, trait_stmt, ctx)?,
            Stmt::StructImpl(impl_stmt) => self.interpret_struct_impl(module, impl_stmt, ctx)?,
            Stmt::TraitImpl(impl_stmt) => self.interpret_trait_impl(module, impl_stmt, ctx)?,
            Stmt::Const(const_stmt) => self.interpret_const(module, const_stmt, ctx, vm)?,
            _ => {}
        }

        Ok(())
    }
}

impl ResolverPass {
    /// Interpret a function declaration.
    ///
    /// # Arguments
    /// * `function` - [`Fn`] - The function to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the function.
    pub fn interpret_function(
        &self,
        module: &mut Module,
        function: roan_ast::Fn,
        ctx: &mut Context,
    ) -> Result<()> {
        debug!("Interpreting function: {}", function.name);

        module.functions.push(StoredFunction::Function {
            function: function.clone(),
            defining_module: module.id(),
        });

        if function.public {
            module.exports.push((
                function.name.clone(),
                ExportType::Function(function.clone()),
            ));
        }

        ctx.upsert_module(module.id().clone(), module.clone());
        Ok(())
    }

    /// Interpret a struct implementation.
    ///
    /// # Arguments
    /// * `impl_stmt` - [`StructImpl`] - The struct implementation to interpret.
    pub fn interpret_struct_impl(
        &mut self,
        module: &mut Module,
        impl_stmt: StructImpl,
        ctx: &mut Context,
    ) -> Result<()> {
        let struct_name = impl_stmt.struct_name.literal();

        let mut found_struct =
            module.get_struct(&struct_name, impl_stmt.struct_name.span.clone())?;
        let stored_impl = StoredImpl {
            def: impl_stmt.clone(),
            defining_module: module.id(),
        };
        found_struct.impls.push(stored_impl.clone());

        if let Some(existing_struct) = module
            .structs
            .iter_mut()
            .find(|s| s.name.literal() == struct_name)
        {
            *existing_struct = found_struct;

            if let Some(export) = module.exports.iter_mut().find(|(n, _)| n == &struct_name) {
                if let ExportType::Struct(s) = &mut export.1 {
                    s.impls.push(stored_impl);
                }
            }
        }

        ctx.upsert_module(module.id().clone(), module.clone());

        Ok(())
    }

    /// Interpret a trait implementation.
    ///
    /// # Arguments
    /// * `impl_stmt` - [`TraitImpl`] - The trait implementation to interpret.
    pub fn interpret_trait_impl(
        &mut self,
        module: &mut Module,
        impl_stmt: TraitImpl,
        ctx: &mut Context,
    ) -> Result<()> {
        let for_name = impl_stmt.struct_name.literal();
        let trait_name = impl_stmt.trait_name.literal();

        let mut struct_def = module.get_struct(&for_name, impl_stmt.struct_name.span.clone())?;
        let trait_def = module.get_trait(&trait_name, impl_stmt.trait_name.span.clone())?;
        if struct_def
            .trait_impls
            .iter()
            .any(|t| t.def.trait_name.literal() == trait_name)
        {
            return Err(RoanError::StructAlreadyImplementsTrait(
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
            return Err(RoanError::TraitMethodNotImplemented(
                trait_name,
                missing_methods,
                impl_stmt.trait_name.span.clone(),
            )
            .into());
        }

        let stored_trait_impl = StoredTraitImpl {
            def: impl_stmt.clone(),
            defining_module: module.id(),
        };

        struct_def.trait_impls.push(stored_trait_impl.clone());

        if let Some(existing_struct) = module
            .structs
            .iter_mut()
            .find(|s| s.name.literal() == for_name)
        {
            *existing_struct = struct_def;

            if let Some(export) = module.exports.iter_mut().find(|(n, _)| n == &for_name) {
                if let ExportType::Struct(s) = &mut export.1 {
                    s.trait_impls.push(stored_trait_impl);
                }
            }
        }

        ctx.upsert_module(module.id().clone(), module.clone());

        Ok(())
    }

    /// Interpret a struct definition.
    ///
    /// # Arguments
    /// * `struct_def` - [`Struct`] - The struct definition to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the struct definition.
    ///
    /// # Returns
    /// The result of interpreting the struct definition.
    pub fn interpret_struct(
        &mut self,
        module: &mut Module,
        struct_stmt: Struct,
        ctx: &mut Context,
    ) -> Result<()> {
        let def = struct_stmt.clone();
        let stored_struct = StoredStruct {
            defining_module: module.id(),
            struct_token: def.struct_token,
            name: def.name,
            fields: def.fields,
            public: def.public,
            impls: vec![],
            trait_impls: vec![],
        };

        module.structs.push(stored_struct.clone());

        if struct_stmt.public {
            module.exports.push((
                struct_stmt.name.literal(),
                ExportType::Struct(stored_struct),
            ));
        }

        ctx.upsert_module(module.id().clone(), module.clone());

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
    pub fn interpret_trait(
        &mut self,
        module: &mut Module,
        trait_stmt: TraitDef,
        ctx: &mut Context,
    ) -> Result<()> {
        module.traits.push(trait_stmt.clone());

        if trait_stmt.public {
            module.exports.push((
                trait_stmt.name.literal(),
                ExportType::Trait(trait_stmt.clone()),
            ));
        }

        ctx.upsert_module(module.id().clone(), module.clone());

        Ok(())
    }

    /// Interpret a const declaration.
    ///
    /// # Arguments
    /// * `const_stmt` - [`Const`] - The const to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the const.
    /// * `vm` - [`VM`] - The virtual machine to use for interpretation.
    ///
    /// # Returns
    /// The result of interpreting the const declaration.
    pub fn interpret_const(
        &mut self,
        module: &mut Module,
        c: Const,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<()> {
        let def_expr = c.expr.clone();
        let ident_literal = c.ident.literal();
        let is_public = c.public;

        module.interpret_expr(&def_expr, ctx, vm)?;

        let val = vm.pop().expect("Expected value on stack");

        let stored_val = StoredConst {
            ident: c.ident.clone(),
            value: val.clone(),
        };

        module.consts.push(stored_val.clone());

        if is_public {
            module
                .exports
                .push((ident_literal, ExportType::Const(stored_val)));
        }

        ctx.upsert_module(module.id().clone(), module.clone());

        Ok(())
    }
}
