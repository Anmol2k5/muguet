// muguet - src/bin/wasm_probe.rs
// Host tool for the Android Wasmtime pipeline (M4). Emitters live in the
// shared `muguet::selftest` module so the device runs identical fixtures.
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
                "mem64shared" => muguet::selftest::emit_mem64shared(),
                _ => muguet::selftest::emit_basic(),
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
