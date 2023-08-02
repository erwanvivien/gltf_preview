#[cfg(feature = "debug_gltf")]
static mut INDENT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[allow(unused)]
#[cfg(feature = "debug_gltf")]
pub(super) fn indent() -> String {
    let indent = unsafe { INDENT.load(std::sync::atomic::Ordering::Relaxed) };
    if indent == 0 {
        return String::new();
    }

    format!("{}└>", " ".repeat(indent - 1))
}

#[allow(unused)]
#[cfg(feature = "debug_gltf")]
pub(super) fn indent_increment() {
    unsafe { INDENT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) };
}

#[allow(unused)]
#[cfg(feature = "debug_gltf")]
pub(super) fn indent_decrement() {
    unsafe { INDENT.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) };
}
