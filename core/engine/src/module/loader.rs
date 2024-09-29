/// Trait that defines the interface for a module loader.
pub trait ModuleLoader {
    /// Load a module from a given source.
    fn load(&self);
}