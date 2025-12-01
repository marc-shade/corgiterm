//! WASM plugin runtime

use std::path::Path;
use wasmtime::*;

/// WASM plugin runtime
pub struct WasmRuntime {
    engine: Engine,
    store: Store<WasmState>,
}

/// WASM plugin state
pub struct WasmState {
    /// Terminal output buffer
    pub output: Vec<String>,
}

impl WasmRuntime {
    pub fn new() -> anyhow::Result<Self> {
        let engine = Engine::default();
        let store = Store::new(&engine, WasmState { output: Vec::new() });

        Ok(Self { engine, store })
    }

    /// Load and execute a WASM module
    pub fn execute(&mut self, path: &Path) -> anyhow::Result<()> {
        let module = Module::from_file(&self.engine, path)?;

        // Create linker with host functions
        let mut linker = Linker::new(&self.engine);

        // Register host functions
        linker.func_wrap(
            "corgiterm",
            "terminal_write",
            |_caller: Caller<'_, WasmState>, ptr: i32, len: i32| {
                tracing::info!("WASM writing to terminal: ptr={}, len={}", ptr, len);
            },
        )?;

        linker.func_wrap(
            "corgiterm",
            "terminal_execute",
            |_caller: Caller<'_, WasmState>, ptr: i32, len: i32| {
                tracing::info!("WASM executing command: ptr={}, len={}", ptr, len);
            },
        )?;

        // Instantiate and run
        let instance = linker.instantiate(&mut self.store, &module)?;

        // Call main/init function if it exists
        if let Ok(main) = instance.get_typed_func::<(), ()>(&mut self.store, "main") {
            main.call(&mut self.store, ())?;
        } else if let Ok(init) = instance.get_typed_func::<(), ()>(&mut self.store, "init") {
            init.call(&mut self.store, ())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_runtime_creation() {
        let runtime = WasmRuntime::new();
        assert!(runtime.is_ok());
    }
}
