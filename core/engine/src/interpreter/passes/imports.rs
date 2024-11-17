use crate::{
    context::Context,
    interpreter::passes::Pass,
    module::{loaders::remove_surrounding_quotes, ExportType, Module, StoredFunction},
    vm::VM,
};
use anyhow::Result;
use roan_ast::{Stmt, Token};
use roan_error::{
    error::RoanError::{FailedToImportModule, ImportError},
    print_diagnostic,
};
use tracing::debug;

#[derive(Clone)]
pub struct ImportPass;

impl Pass for ImportPass {
    fn pass_stmt(
        &mut self,
        stmt: Stmt,
        module: &mut Module,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<()> {
        match stmt {
            Stmt::Use(u) => {
                debug!("Interpreting use: {}", u.from.literal());

                let mut loaded_module = ctx
                    .load_module(
                        &module.clone(),
                        remove_surrounding_quotes(&u.from.literal()),
                    )
                    .map_err(|err| {
                        FailedToImportModule(
                            u.from.literal().to_string(),
                            err.to_string(),
                            u.from.span.clone(),
                        )
                    })?;

                match loaded_module.parse(ctx, vm) {
                    Ok(_) => {}
                    Err(e) => {
                        print_diagnostic(&e, Some(loaded_module.source().content()), module.path());
                        std::process::exit(1);
                    }
                }

                // Collect the items to import
                let imported_items: Vec<(String, &Token)> =
                    u.items.iter().map(|i| (i.literal(), i)).collect();

                Ok(for (name, item) in imported_items {
                    let export = loaded_module.exports.iter().find(|(n, _)| n == &name);

                    if let Some((name, value)) = export {
                        debug!("Importing {} from {}", name, u.from.literal());
                        match value {
                            ExportType::Function(f) => {
                                module.functions.push(StoredFunction::Function {
                                    function: f.clone(),
                                    defining_module: loaded_module.id(),
                                });
                            }
                            ExportType::Struct(s) => {
                                module.structs.push(s.clone());
                            }
                            ExportType::Trait(t) => {
                                module.traits.push(t.clone());
                            }
                            ExportType::Const(c) => {
                                module.consts.push(c.clone());
                            }
                        }
                    } else {
                        return Err(ImportError(name, item.span.clone()).into());
                    }
                })
            }
            _ => Ok(()),
        }
    }
}
