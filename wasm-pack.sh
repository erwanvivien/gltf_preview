WEBGPU_FLAGS="--cfg=web_sys_unstable_apis"
CARGO_MODE=""
TARGET_PATH="debug"

case "$*" in
  *-webgl*)
    WEBGPU_FLAGS=""
    ;;
  *-r*) # -r, --release
    CARGO_MODE="--release"
    TARGET_PATH="release"
    ;;
esac

echo "Building with webgpu: ${WEBGPU_FLAGS}"
echo "Building with cargo mode: ${CARGO_MODE:-"--debug"}"

RUSTFLAGS="${WEBGPU_FLAGS}" \
    cargo build ${CARGO_MODE} \
    --target wasm32-unknown-unknown \
    --features wasm

wasm-bindgen --out-dir mario_skurt \
    --web target/wasm32-unknown-unknown/${TARGET_PATH}/mario_skurt.wasm
