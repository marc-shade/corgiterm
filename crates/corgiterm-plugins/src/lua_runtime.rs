//! Lua scripting runtime

use mlua::{Lua, Result as LuaResult};
use std::path::Path;

/// Lua plugin runtime
pub struct LuaRuntime {
    lua: Lua,
}

impl LuaRuntime {
    pub fn new() -> LuaResult<Self> {
        // Create Lua instance
        // Note: For sandboxing, we could use Lua::new_with() with specific StdLib flags
        // but for now use the default safe configuration
        let lua = Lua::new();

        // Register CorgiTerm API
        Self::register_api(&lua)?;

        Ok(Self { lua })
    }

    fn register_api(lua: &Lua) -> LuaResult<()> {
        let globals = lua.globals();

        // Create corgiterm table
        let corgi = lua.create_table()?;

        // Terminal functions
        let terminal = lua.create_table()?;

        terminal.set(
            "write",
            lua.create_function(|_, text: String| {
                tracing::info!("Lua plugin writing: {}", text);
                Ok(())
            })?,
        )?;

        terminal.set(
            "execute",
            lua.create_function(|_, command: String| {
                tracing::info!("Lua plugin executing: {}", command);
                Ok(())
            })?,
        )?;

        corgi.set("terminal", terminal)?;

        // UI functions
        let ui = lua.create_table()?;

        ui.set(
            "notify",
            lua.create_function(|_, (title, message): (String, String)| {
                tracing::info!("Lua plugin notification: {} - {}", title, message);
                Ok(())
            })?,
        )?;

        corgi.set("ui", ui)?;

        globals.set("corgiterm", corgi)?;

        Ok(())
    }

    /// Execute a Lua script
    pub fn execute(&self, script: &str) -> LuaResult<()> {
        self.lua.load(script).exec()
    }

    /// Execute a Lua file
    pub fn execute_file(&self, path: &Path) -> LuaResult<()> {
        let script = std::fs::read_to_string(path)
            .map_err(|e| mlua::Error::ExternalError(std::sync::Arc::new(e)))?;
        self.execute(&script)
    }

    /// Call a function in the loaded script
    pub fn call_function(&self, name: &str, args: impl mlua::IntoLuaMulti) -> LuaResult<()> {
        let globals = self.lua.globals();
        let func: mlua::Function = globals.get(name)?;
        func.call::<()>(args)?;
        Ok(())
    }
}

impl Default for LuaRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create Lua runtime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_runtime_creation() {
        let runtime = LuaRuntime::new().unwrap();
        runtime.execute("print('Hello from Lua')").unwrap();
    }

    #[test]
    fn test_corgiterm_api() {
        let runtime = LuaRuntime::new().unwrap();
        runtime.execute("corgiterm.terminal.write('test')").unwrap();
    }
}
