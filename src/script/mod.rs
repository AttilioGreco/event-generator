pub mod functions;

use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use mlua::{HookTriggers, Lua, LuaOptions, StdLib};

pub struct ScriptEngine {
    code: String,
    max_instructions: u32,
}

impl ScriptEngine {
    pub fn from_file(path: &str, max_instructions: u32) -> Result<Self> {
        let code = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read script file: {path}"))?;
        Self::validate(&code)
            .with_context(|| format!("failed to compile script file: {path}"))?;
        Ok(Self { code, max_instructions })
    }

    pub fn from_inline(code: &str, max_instructions: u32) -> Result<Self> {
        Self::validate(code).with_context(|| "failed to compile inline script")?;
        Ok(Self { code: code.to_string(), max_instructions })
    }

    fn validate(code: &str) -> Result<()> {
        let lua = Self::create_lua(0)?;
        lua.load(code)
            .into_function()
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("syntax error: {e}"))
    }

    fn create_lua(max_instructions: u32) -> Result<Lua> {
        let lua = Lua::new_with(
            StdLib::STRING | StdLib::TABLE | StdLib::MATH,
            LuaOptions::default(),
        )
        .map_err(|e| anyhow::anyhow!("failed to create Lua state: {e}"))?;

        if max_instructions > 0 {
            lua.set_hook(
                HookTriggers {
                    every_nth_instruction: Some(max_instructions),
                    ..Default::default()
                },
                |_lua, _debug| {
                    Err(mlua::Error::RuntimeError(
                        "script exceeded instruction limit".into(),
                    ))
                },
            );
        }

        functions::register_all(&lua)
            .map_err(|e| anyhow::anyhow!("failed to register script functions: {e}"))?;

        Ok(lua)
    }

    /// Execute the script once, returning all emitted lines joined by `\n`.
    pub fn run(&self) -> String {
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let lua = match Self::create_lua(self.max_instructions) {
            Ok(l) => l,
            Err(e) => return format!("SCRIPT_ERROR: {e}"),
        };

        let buf = Arc::clone(&buffer);
        let emit_fn = match lua.create_function(move |_, line: String| {
            buf.lock().unwrap().push(line);
            Ok(())
        }) {
            Ok(f) => f,
            Err(e) => return format!("SCRIPT_ERROR: {e}"),
        };

        if let Err(e) = lua.globals().set("emit", emit_fn) {
            return format!("SCRIPT_ERROR: {e}");
        }

        match lua.load(&self.code).exec() {
            Ok(()) => {}
            Err(e) => {
                eprintln!("[script] runtime error: {e}");
                return format!("SCRIPT_ERROR: {e}");
            }
        }

        let lines = buffer.lock().unwrap();
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_emit() {
        let engine = ScriptEngine::from_inline(r#"emit("hello world")"#, 100_000).unwrap();
        let output = engine.run();
        assert_eq!(output, "hello world");
    }

    #[test]
    fn multiple_emits_joined() {
        let engine = ScriptEngine::from_inline(
            r#"
            emit("line one")
            emit("line two")
            emit("line three")
            "#,
            100_000,
        )
        .unwrap();
        let output = engine.run();
        assert_eq!(output, "line one\nline two\nline three");
    }

    #[test]
    fn emit_with_interpolation() {
        let engine = ScriptEngine::from_inline(
            r#"
            local ip = fake_ipv4()
            local user = fake_username()
            emit("user=" .. user .. " ip=" .. ip)
            "#,
            100_000,
        )
        .unwrap();
        let output = engine.run();
        assert!(output.contains("user="), "missing user: {output}");
        assert!(output.contains("ip="), "missing ip: {output}");
    }

    #[test]
    fn conditional_branching() {
        let engine = ScriptEngine::from_inline(
            r#"
            if weighted_bool(1.0) then
                emit("success")
            else
                emit("failure")
            end
            "#,
            100_000,
        )
        .unwrap();
        let output = engine.run();
        assert_eq!(output, "success");
    }

    #[test]
    fn multiline_trace() {
        let engine = ScriptEngine::from_inline(
            r#"
            local req = uuid()
            emit("START req=" .. req)
            for i = 0, 2 do
                emit("  processing step " .. i)
            end
            emit("END req=" .. req)
            "#,
            100_000,
        )
        .unwrap();
        let output = engine.run();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 5, "expected 5 lines: {output}");
        let req_start = lines[0].split("req=").nth(1).unwrap();
        let req_end = lines[4].split("req=").nth(1).unwrap();
        assert_eq!(req_start, req_end, "req_id mismatch");
    }

    #[test]
    fn max_instructions_prevents_infinite_loop() {
        let engine = ScriptEngine::from_inline(
            r#"
            while true do
                emit("spam")
            end
            "#,
            1_000,
        )
        .unwrap();
        let output = engine.run();
        assert!(
            output.starts_with("SCRIPT_ERROR:"),
            "expected error for infinite loop: {output}"
        );
    }

    #[test]
    fn compile_error_caught() {
        let result = ScriptEngine::from_inline("this is not valid lua {{{}}", 100_000);
        assert!(result.is_err());
    }

    #[test]
    fn empty_script_produces_empty_output() {
        let engine = ScriptEngine::from_inline("local x = 1", 100_000).unwrap();
        let output = engine.run();
        assert!(output.is_empty());
    }

    #[test]
    fn pick_and_functions_work_in_script() {
        let engine = ScriptEngine::from_inline(
            r#"
            local level = fake_log_level()
            local method = fake_http_method()
            local status = fake_http_status()
            emit(level .. " " .. method .. " " .. status)
            "#,
            100_000,
        )
        .unwrap();
        let output = engine.run();
        assert!(!output.is_empty(), "output should not be empty");
        let parts: Vec<&str> = output.split_whitespace().collect();
        assert_eq!(parts.len(), 3, "expected 3 parts: {output}");
    }
}
