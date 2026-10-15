// muguet - src/bin/wasm_probe.rs
// Generates tiny WASM fixtures for the Android Wasmtime pipeline (M4) and
// verifies that cross-compiled artifacts are triple-bound.
// Usage:
//   wasm_probe emit <out.wasm> [basic|mem64shared]
//   wasm_probe check-mismatch <module.cwasm>   (host deserialize must FAIL)

#[cfg(feature = "jit")]
fn main() -> anyhow::Result<()> {
    use anyhow::Context as _;
    let argv: Vec<String> = std::env::args().collect();
    match argv.get(1).map(String::as_str) {
        Some("emit") => {
            let out = argv.get(2).map(String::as_str).unwrap_or("probe.wasm");
            let kind = argv.get(3).map(String::as_str).unwrap_or("basic");
            let bytes = match kind {
                "mem64shared" => emit_mem64shared()?,
                _ => emit_basic()?,
            };
            std::fs::write(out, &bytes)?;
            eprintln!("[probe] wrote {} ({} bytes, {kind})", out, bytes.len());
            Ok(())
        }
        Some("check-mismatch") => {
            let path = argv.get(2).context("--check-mismatch <module.cwasm>")?;
            let bytes = std::fs::read(path)?;
            let engine = wasmtime::Engine::new(&wasmtime::Config::new())?;
            match unsafe { wasmtime::Module::deserialize(&engine, &bytes) } {
                Ok(_) => {
                    eprintln!("[probe] UNEXPECTED: host engine accepted the artifact");
                    std::process::exit(2);
                }
                Err(e) => {
                    eprintln!("[probe] host deserialize rejected artifact (expected): {e:#}");
                    Ok(())
                }
            }
        }
        _ => {
            eprintln!("usage: wasm_probe emit <out.wasm> [basic|mem64shared]");
            eprintln!("       wasm_probe check-mismatch <module.cwasm>");
            std::process::exit(1);
        }
    }
}

#[cfg(not(feature = "jit"))]
fn main() {
    eprintln!("wasm_probe needs the `jit` feature");
    std::process::exit(1);
}

/// `(module (func (export "add") (param i32 i32) (result i32) local.get 0 local.get 1 i32.add))`
#[cfg(feature = "jit")]
fn emit_basic() -> anyhow::Result<Vec<u8>> {
    use wasm_encoder::{CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module, TypeSection, ValType};
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32, ValType::I32], [ValType::I32]);
    module.section(&types);
    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);
    let mut exports = ExportSection::new();
    exports.export("add", ExportKind::Func, 0);
    module.section(&exports);
    let mut codes = CodeSection::new();
    let mut f = Function::new([]);
    f.instruction(&Instruction::LocalGet(0));
    f.instruction(&Instruction::LocalGet(1));
    f.instruction(&Instruction::I32Add);
    f.instruction(&Instruction::End);
    codes.function(&f);
    module.section(&codes);
    Ok(module.finish())
}

/// `(module (memory (export "mem") 1 256 i64 shared) (func (export "grow") ...))`
/// Exercises the memory64 + shared-memory combination Muguet requires.
#[cfg(feature = "jit")]
fn emit_mem64shared() -> anyhow::Result<Vec<u8>> {
    use wasm_encoder::{
        CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, MemorySection,
        MemoryType, Module, TypeSection, ValType,
    };
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I64], [ValType::I64]);
    module.section(&types);
    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);
    let mut mems = MemorySection::new();
    mems.memory(MemoryType {
        minimum: 1,
        maximum: Some(256),
        memory64: true,
        shared: true,
        page_size_log2: None,
    });
    module.section(&mems);
    let mut exports = ExportSection::new();
    exports.export("mem", ExportKind::Memory, 0);
    exports.export("grow", ExportKind::Func, 0);
    module.section(&exports);
    let mut codes = CodeSection::new();
    let mut f = Function::new([]);
    f.instruction(&Instruction::LocalGet(0));
    f.instruction(&Instruction::MemoryGrow(0));
    f.instruction(&Instruction::End);
    codes.function(&f);
    module.section(&codes);
    Ok(module.finish())
}
