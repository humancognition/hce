use std::process::Command;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let wasm_path = std::path::Path::new(&out_dir)
        .join("../../../")
        .canonicalize()
        .unwrap()
        .join("target/wasm32-unknown-unknown/release/hce_wasm.wasm");

    if !wasm_path.exists() {
        return;
    }

    let optimized = wasm_path.with_file_name("hce_wasm_opt.wasm");

    let status = Command::new("wasm-opt")
        .args(["-Os", "--enable-bulk-memory"])
        .arg(&wasm_path)
        .arg("-o")
        .arg(&optimized)
        .status();

    match status {
        Ok(s) if s.success() => {
            std::fs::rename(&optimized, &wasm_path).ok();
        }
        _ => {}
    }

    println!("cargo:rerun-if-changed=build.rs");
}
