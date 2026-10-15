// muguet - selftest.rs
// Device-executable smoke tests that need no game assets (M4/M5 groundwork).
// Trigger: `--selftest=wasm|vulkan|all` on desktop tools, or a `SELFTEST`
// file containing one of those words in the support dir on Android
// (created via adb push / Device File Explorer).
//
// The WASM tests run on the REAL engine policy (engine_config_for), so they
// exercise memory64 + shared memory + atomics + host imports + traps exactly
// as the game will. The Vulkan test brings up a headless adapter+device with
// no surface. Results go to stderr AND logcat (alog) on Android.
//
// What this does NOT prove (honest scope): real game module init, WASM thread
// spawning (needs the emscripten thread runtime from game code), swapchain
// presentation, or any FPS/memory figure. Those need a device + game assets.

use anyhow::{Context, Result};
use wasmtime::*;

pub fn requested(argv: &[String], _support_dir: &std::path::Path) -> Option<String> {
    for a in argv.iter().skip(1) {
        if let Some(kind) = a.strip_prefix("--selftest=") {
            return Some(kind.to_string());
        }
    }
    #[cfg(target_os = "android")]
    if let Some(kind) = requested_from_marker(_support_dir) {
        return Some(kind);
    }
    None
}

/// Android trigger for asset-independent self-tests: a `SELFTEST` file holding
/// `wasm`, `vulkan` or `all` in the support dir (placed via `run-as` + push or
/// Device File Explorer). Pure filesystem logic so it unit-tests anywhere.
fn requested_from_marker(support_dir: &std::path::Path) -> Option<String> {
    let kind = std::fs::read_to_string(support_dir.join("SELFTEST")).ok()?;
    let kind = kind.trim().to_lowercase();
    ["wasm", "vulkan", "all"].contains(&kind.as_str()).then(|| kind)
}

pub fn run(kind: &str) -> Result<()> {
    tlog(&format!("selftest start ({kind})"));
    match kind {
        "wasm" => run_wasm()?,
        "vulkan" => run_vulkan()?,
        "all" => {
            run_wasm()?;
            run_vulkan()?;
        }
        other => anyhow::bail!("unknown selftest kind: {other} (want wasm|vulkan|all)"),
    }
    tlog("selftest PASS");
    Ok(())
}

fn tlog(s: &str) {
    eprintln!("[selftest] {s}");
    #[cfg(target_os = "android")]
    crate::alog::i("selftest", s);
}

/// anyhow context for wasmtime results (matches the crate's
/// `.map_err(anyhow::Error::from)` convention in lib.rs).
trait Wctx<T> {
    fn wctx(self, what: &str) -> anyhow::Result<T>;
}

impl<T> Wctx<T> for Result<T, wasmtime::Error> {
    fn wctx(self, what: &str) -> anyhow::Result<T> {
        self.map_err(anyhow::Error::from).with_context(|| format!("selftest: {what}"))
    }
}

fn tbail(what: &str) -> anyhow::Error {
    let msg = format!("selftest FAIL: {what}");
    eprintln!("[selftest] {msg}");
    #[cfg(target_os = "android")]
    crate::alog::e("selftest", &msg);
    anyhow::anyhow!("{msg}")
}

// ---------------------------------------------------------------- WASM ---

fn test_engine() -> Result<Engine> {
    Engine::new(&crate::engine_config_for(crate::mobile_memory())).map_err(anyhow::Error::from)
}

fn run_wasm() -> Result<()> {
    // 1. Basic arithmetic through a compiled module.
    {
        let engine = test_engine()?;
        let module = Module::new(&engine, emit_basic()).wctx("compile basic")?;
        let mut store = Store::new(&engine, ());
        let instance = Linker::new(&engine).instantiate(&mut store, &module).wctx("instantiate basic")?;
        let add = instance.get_typed_func::<(i32, i32), i32>(&mut store, "add").wctx("missing export add")?;
        let got = add.call(&mut store, (40, 2)).wctx("call add")?;
        if got != 42 {
            return Err(tbfail("basic add", &format!("want 42, got {got}")));
        }
        tlog("wasm/basic add(40,2)=42 ok");
    }
    // 2. memory64 + shared memory: size, growth, atomic RMW in-guest.
    {
        let engine = test_engine()?;
        let mem_ty = MemoryTypeBuilder::new()
            .min(1)
            .max(Some(256))
            .memory64(true)
            .shared(true)
            .build()
            .map_err(anyhow::Error::from)?;
        let mem = SharedMemory::new(&engine, mem_ty).map_err(|_| tbfail("mem64shared", "SharedMemory::new rejected"))?;
        let mut store = Store::new(&engine, ());
        if mem.data_size() != 65536 {
            return Err(tbfail("mem64shared", "initial size != 64KiB"));
        }
        mem.grow(1).map_err(|_| tbfail("mem64shared", "grow(1) failed"))?;
        if mem.data_size() != 131072 {
            return Err(tbfail("mem64shared", "grown size != 128KiB"));
        }
        let module = Module::new(&engine, emit_mem64shared()).wctx("compile mem64shared")?;
        let mut linker = Linker::new(&engine);
        linker.define(&store, "env", "memory", mem.clone()).wctx("define env.memory")?;
        let instance = linker.instantiate(&mut store, &module).wctx("instantiate mem64shared")?;
        let atom = instance
            .get_typed_func::<(i64, i64), i64>(&mut store, "atom")
            .wctx("missing export atom")?;
        // I64AtomicAdd returns the previous value.
        if atom.call(&mut store, (0, 5)).wctx("call atom")? != 0 {
            return Err(tbfail("mem64shared", "first atomic add did not return 0"));
        }
        if atom.call(&mut store, (0, 7)).wctx("call atom")? != 5 {
            return Err(tbfail("mem64shared", "second atomic add did not return 5"));
        }
        tlog("wasm/mem64+shared+grow+atomics ok");
    }
    // 3. Host import called from guest code.
    {
        let engine = test_engine()?;
        let module = Module::new(&engine, emit_hostuser()).wctx("compile hostuser")?;
        let mut linker = Linker::new(&engine);
        linker
            .func_new("env", "host_add", FuncType::new(&engine, [ValType::I32, ValType::I32], [ValType::I32]), |_, params, results| {
                let a = params[0].i32().unwrap_or(0);
                let b = params[1].i32().unwrap_or(0);
                results[0] = Val::I32(a + b);
                Ok(())
            })
            .wctx("define host_add")?;
        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module).wctx("instantiate hostuser")?;
        let f = instance.get_typed_func::<(), i32>(&mut store, "usehost").wctx("missing export usehost")?;
        if f.call(&mut store, ()).wctx("call usehost")? != 15 {
            return Err(tbfail("hostuser", "host_add(7,8) != 15 through guest"));
        }
        tlog("wasm/host import ok");
    }
    // 4. Trap propagation (unreachable must surface as an error, not abort).
    {
        let engine = test_engine()?;
        let module = Module::new(&engine, emit_trap()).wctx("compile trap")?;
        let mut store = Store::new(&engine, ());
        let instance = Linker::new(&engine).instantiate(&mut store, &module).wctx("instantiate trap")?;
        let f = instance.get_typed_func::<(), ()>(&mut store, "boom").wctx("missing export boom")?;
        match f.call(&mut store, ()) {
            Ok(()) => return Err(tbfail("trap", "unreachable returned Ok")),
            Err(e) => {
                let s = format!("{e:?}");
                if !s.contains("unreachable") {
                    return Err(tbfail("trap", &format!("wrong trap: {s}")));
                }
            }
        }
        tlog("wasm/trap propagation ok");
    }
    // NOTE: guest thread spawning (__pthread_create_js over shared memory) is
    // only exercised by real game init (M6): it needs the emscripten thread
    // runtime inside game code. Shared memory + atomics above are its
    // foundation and they pass here.
    Ok(())
}

fn tbfail(test: &str, what: &str) -> anyhow::Error {
    tbail(&format!("{test}: {what}"))
}

// ------------------------------------------------------------- Vulkan ---

fn run_vulkan() -> Result<()> {
    use wgpu::*;
    let mut desc = InstanceDescriptor::new_without_display_handle();
    desc.backends = crate::gpu::NATIVE_BACKENDS;
    let instance = Instance::new(desc);
    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }))
    .map_err(|e| tbail(&format!("no adapter: {e:?}")))?;
    let info = adapter.get_info();
    tlog(&format!("vulkan/adapter: {} ({:?}) driver={}", info.name, info.backend, info.driver));
    if info.backend != Backend::Vulkan && cfg!(target_os = "android") {
        return Err(tbfail("adapter", &format!("backend is {:?}, want Vulkan", info.backend)));
    }
    let features = Features::empty();
    let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
        label: Some("muguet-selftest"),
        required_features: features,
        required_limits: Limits::default(),
        ..Default::default()
    }))
    .map_err(|e| tbail(&format!("device creation failed: {e:?}")))?;
    let buf = device.create_buffer(&BufferDescriptor {
        label: Some("selftest"),
        size: 256,
        usage: BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    drop(buf);
    queue.submit([]);
    device
        .poll(PollType::Wait { submission_index: None, timeout: None })
        .map_err(|e| tbail(&format!("queue poll failed: {e:?}")))?;
    tlog("vulkan/headless adapter+device+submit ok (no surface: presentation UNTESTED)");
    Ok(())
}

// ------------------------------------------------------------ emitters ---

/// `(module (func (export "add") (param i32 i32) (result i32) ...))`
pub fn emit_basic() -> Vec<u8> {
    use wasm_encoder::*;
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
    module.finish()
}

/// Shared memory64 module with an exported atomic add:
/// `(module (import "env" "memory" (memory 1 256 i64 shared))
///   (func (export "atom") (param i64 i64) (result i64)
///     local.get 0 local.get 1 i64.atomic.rmw.add))`
pub fn emit_mem64shared() -> Vec<u8> {
    use wasm_encoder::*;
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I64, ValType::I64], [ValType::I64]);
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import(
        "env",
        "memory",
        MemoryType { minimum: 1, maximum: Some(256), memory64: true, shared: true, page_size_log2: None },
    );
    module.section(&imports);
    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);
    let mut exports = ExportSection::new();
    exports.export("atom", ExportKind::Func, 0);
    module.section(&exports);
    let mut codes = CodeSection::new();
    let mut f = Function::new([]);
    f.instruction(&Instruction::LocalGet(0));
    f.instruction(&Instruction::LocalGet(1));
    f.instruction(&Instruction::I64AtomicRmwAdd(MemArg { offset: 0, align: 3, memory_index: 0 }));
    f.instruction(&Instruction::End);
    codes.function(&f);
    module.section(&codes);
    module.finish()
}

/// `(module (import "env" "host_add" (func (param i32 i32) (result i32)))
///   (func (export "usehost") (result i32) i32.const 7 i32.const 8 call 0))`
fn emit_hostuser() -> Vec<u8> {
    use wasm_encoder::*;
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32, ValType::I32], [ValType::I32]);
    types.ty().function([], [ValType::I32]);
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import("env", "host_add", EntityType::Function(0));
    module.section(&imports);
    let mut funcs = FunctionSection::new();
    funcs.function(1);
    module.section(&funcs);
    let mut exports = ExportSection::new();
    exports.export("usehost", ExportKind::Func, 1);
    module.section(&exports);
    let mut codes = CodeSection::new();
    let mut f = Function::new([]);
    f.instruction(&Instruction::I32Const(7));
    f.instruction(&Instruction::I32Const(8));
    f.instruction(&Instruction::Call(0));
    f.instruction(&Instruction::End);
    codes.function(&f);
    module.section(&codes);
    module.finish()
}

/// `(module (func (export "boom") unreachable))`
fn emit_trap() -> Vec<u8> {
    use wasm_encoder::*;
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([], []);
    module.section(&types);
    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);
    let mut exports = ExportSection::new();
    exports.export("boom", ExportKind::Func, 0);
    module.section(&exports);
    let mut codes = CodeSection::new();
    let mut f = Function::new([]);
    f.instruction(&Instruction::Unreachable);
    f.instruction(&Instruction::End);
    codes.function(&f);
    module.section(&codes);
    module.finish()
}

#[cfg(test)]
mod tests {
    /// Fixtures must be well-formed under the exact feature set the engine
    /// enables (threads/memory64/multi-memory all default-true in wasmparser).
    #[test]
    fn fixtures_validate() {
        for m in [super::emit_basic(), super::emit_mem64shared()] {
            wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::default())
                .validate_all(&m)
                .expect("selftest fixture must validate");
        }
    }

    /// CLI trigger works on every host; unknown kinds pass through for run()
    /// to reject with a clear error (never silently ignored).
    #[test]
    fn cli_trigger_parsing() {
        let argv = vec!["muguet".to_string(), "--selftest=vulkan".to_string()];
        assert_eq!(super::requested(&argv, std::path::Path::new("/none")).as_deref(), Some("vulkan"));
        let argv = vec!["muguet".to_string()];
        assert_eq!(super::requested(&argv, std::path::Path::new("/none")), None);
        let argv = vec!["muguet".to_string(), "--selftest=bogus".to_string()];
        assert_eq!(super::requested(&argv, std::path::Path::new("/none")).as_deref(), Some("bogus"));
    }

    /// Marker-file trigger (Android SELFTEST): valid kinds accepted
    /// case-insensitively, anything else ignored, missing file is None.
    #[test]
    fn marker_trigger_parsing() {
        let dir = std::env::temp_dir().join(format!("muguet-marker-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(super::requested_from_marker(&dir), None);
        std::fs::write(dir.join("SELFTEST"), "VuLkAn\n").unwrap();
        assert_eq!(super::requested_from_marker(&dir).as_deref(), Some("vulkan"));
        std::fs::write(dir.join("SELFTEST"), "nonsense").unwrap();
        assert_eq!(super::requested_from_marker(&dir), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
