// muguet - alog.rs (Android only)
// Logcat logging via __android_log_write. liblog.so is already a linked
// dependency of the Android binary (see llvm-readelf NEEDED output).
// Rust stderr is NOT reliably visible under NativeActivity, so every fatal or
// lifecycle event on Android must go through here in addition to eprintln!.
// Panics are also routed here via init(): without that hook a native panic is
// an unexplained silent death.

use std::ffi::CString;
use std::os::raw::{c_char, c_int};

#[allow(dead_code)]
const DEBUG: c_int = 3;
const INFO: c_int = 4;
#[allow(dead_code)]
const WARN: c_int = 5;
const ERROR: c_int = 6;

/// Logcat tag. Kept short: pre-Android-8 truncates tags past 23 chars.
const TAG: &str = "muguet";

extern "C" {
    fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
}

fn write(prio: c_int, cat: &str, msg: &str) {
    let tag = CString::new(TAG).unwrap_or_default();
    // Logcat truncates lines past ~4000 bytes; chunk defensively.
    let full = format!("[{cat}] {msg}");
    let bytes = full.as_bytes();
    for chunk in bytes.chunks(3000) {
        let text = String::from_utf8_lossy(chunk).into_owned();
        if let Ok(c) = CString::new(text) {
            unsafe {
                __android_log_write(prio, tag.as_ptr(), c.as_ptr());
            }
        }
    }
}

#[allow(dead_code)]
pub fn d(cat: &str, msg: &str) {
    write(DEBUG, cat, msg);
}
pub fn i(cat: &str, msg: &str) {
    write(INFO, cat, msg);
}
#[allow(dead_code)]
pub fn w(cat: &str, msg: &str) {
    write(WARN, cat, msg);
}
pub fn e(cat: &str, msg: &str) {
    write(ERROR, cat, msg);
}

/// Route panics to logcat and install nothing else. Must be called first in
/// android_main, before any fallible work.
pub fn init() {
    std::panic::set_hook(Box::new(|info| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
            .unwrap_or("<non-string panic>");
        let loc = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_else(|| "?".into());
        e("panic", &format!("{loc}: {payload}"));
    }));
    i("lifecycle", "logcat logging online");
}
