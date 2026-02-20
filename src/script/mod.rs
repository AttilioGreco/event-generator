pub mod functions;

use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rhai::{AST, Engine, Scope};

pub struct ScriptEngine {
    ast: AST,
    max_operations: u64,
}

impl ScriptEngine {
    pub fn from_file(path: &str, max_operations: u64) -> Result<Self> {
        let code = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read script file: {path}"))?;
        Self::build(&code, max_operations)
            .with_context(|| format!("failed to compile script file: {path}"))
    }

    pub fn from_inline(code: &str, max_operations: u64) -> Result<Self> {
        Self::build(code, max_operations).with_context(|| "failed to compile inline script")
    }

    fn build(code: &str, max_operations: u64) -> Result<Self> {
        let mut engine = Self::create_engine(max_operations);

        // Register a dummy emit for compilation (signature check only)
        engine.register_fn("emit", |_line: &str| {});

        let ast = engine.compile(code)?;

        Ok(Self {
            ast,
            max_operations,
        })
    }

    fn create_engine(max_operations: u64) -> Engine {
        let mut engine = Engine::new();

        // Sandboxing limits
        engine.set_max_operations(max_operations);
        engine.set_max_call_levels(32);
        engine.set_max_string_size(65_536);
        engine.set_max_array_size(10_000);
        engine.set_max_map_size(1_000);

        // Register all custom functions (except emit)
        functions::register_all(&mut engine);

        engine
    }

    /// Execute the script once (one "step"), returning all emitted lines joined by `\n`.
    pub fn run(&self) -> String {
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let mut engine = Self::create_engine(self.max_operations);

        // Register emit() with the per-run buffer
        let buf = Arc::clone(&buffer);
        engine.register_fn("emit", move |line: &str| {
            buf.lock().unwrap().push(line.to_string());
        });

        let mut scope = Scope::new();

        match engine.run_ast_with_scope(&mut scope, &self.ast) {
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
        let engine = ScriptEngine::from_inline(r#"emit("hello world");"#, 10_000).unwrap();
        let output = engine.run();
        assert_eq!(output, "hello world");
    }

    #[test]
    fn multiple_emits_joined() {
        let engine = ScriptEngine::from_inline(
            r#"
            emit("line one");
            emit("line two");
            emit("line three");
            "#,
            10_000,
        )
        .unwrap();
        let output = engine.run();
        assert_eq!(output, "line one\nline two\nline three");
    }

    #[test]
    fn emit_with_interpolation() {
        let engine = ScriptEngine::from_inline(
            r#"
            let ip = fake_ipv4();
            let user = fake_username();
            emit("user=" + user + " ip=" + ip);
            "#,
            10_000,
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
            if weighted_bool(1.0) {
                emit("success");
            } else {
                emit("failure");
            }
            "#,
            10_000,
        )
        .unwrap();
        let output = engine.run();
        assert_eq!(output, "success");
    }

    #[test]
    fn multiline_trace() {
        let engine = ScriptEngine::from_inline(
            r#"
            let req = uuid();
            emit("START req=" + req);
            for i in 0..3 {
                emit("  processing step " + i);
            }
            emit("END req=" + req);
            "#,
            10_000,
        )
        .unwrap();
        let output = engine.run();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 5, "expected 5 lines: {output}");
        // Verify req_id consistency
        let req_start = lines[0].split("req=").nth(1).unwrap();
        let req_end = lines[4].split("req=").nth(1).unwrap();
        assert_eq!(req_start, req_end, "req_id mismatch");
    }

    #[test]
    fn max_operations_prevents_infinite_loop() {
        let engine = ScriptEngine::from_inline(
            r#"
            loop {
                emit("spam");
            }
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
        let result = ScriptEngine::from_inline("this is not valid rhai {{{}}", 10_000);
        assert!(result.is_err());
    }

    #[test]
    fn empty_script_produces_empty_output() {
        let engine = ScriptEngine::from_inline("let x = 1;", 10_000).unwrap();
        let output = engine.run();
        assert!(output.is_empty());
    }

    #[test]
    fn pick_and_functions_work_in_script() {
        let engine = ScriptEngine::from_inline(
            r#"
            let level = fake_log_level();
            let method = fake_http_method();
            let status = fake_http_status();
            emit(level + " " + method + " " + status);
            "#,
            10_000,
        )
        .unwrap();
        let output = engine.run();
        assert!(!output.is_empty(), "output should not be empty");
        let parts: Vec<&str> = output.split_whitespace().collect();
        assert_eq!(parts.len(), 3, "expected 3 parts: {output}");
    }
}
