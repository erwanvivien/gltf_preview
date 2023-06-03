$WEBGPU_FLAGS="--cfg=web_sys_unstable_apis"
$CARGO_MODE=""
$TARGET_PATH="debug"

if ($args -match "-webgl") {
    $WEBGPU_FLAGS=""
}
if ($args -match "-r") { # -r, --release
    $CARGO_MODE="--release"
    $TARGET_PATH="release"
}

Write-Host "Building with webgpu: ${WEBGPU_FLAGS}"
Write-Host "Building with cargo mode: ${CARGO_MODE}"

$PREVIOUS_RUSTFLAGS = $env:RUSTFLAGS
$env:RUSTFLAGS="${WEBGPU_FLAGS}"
& cargo build ${CARGO_MODE} `
    --target wasm32-unknown-unknown `
    --features wasm

& wasm-bindgen --out-dir mario_skurt `
    --web target/wasm32-unknown-unknown/${TARGET_PATH}/mario_skurt.wasm

$env:RUSTFLAGS = $PREVIOUS_RUSTFLAGS
