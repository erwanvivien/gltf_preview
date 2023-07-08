#[cfg(target_arch = "wasm32")]
use web_sys::{Request, Response};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::*, JsValue};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
async fn fetch_file<P: AsRef<std::path::Path>>(path: P) -> Result<Response, JsValue> {
    let path = path.as_ref().to_str().unwrap();

    const BASE_URL: &str = "http://localhost:3000";
    let url = format!("{}/{}", BASE_URL, path);
    dbg!(&url);

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_str(&url)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    Ok(resp)
}

#[cfg(target_arch = "wasm32")]
/// Load a file from the a server URI. \
/// load_file_string("scene.gltf") -> "http://localhost:3000/scene.gltf"
///
/// # Errors
///
/// Returns an error if the file cannot be downloaded.
pub async fn load_file_buffer<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<u8>, JsValue> {
    let resp = fetch_file(path).await?;

    // Convert this other `Promise` into a rust `Future`.
    let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
    let u8_array: js_sys::Uint8Array = js_sys::Uint8Array::new(&array_buffer);

    // Send the JSON response back to JS.
    Ok(u8_array.to_vec())
}

/// Load a file from the filesystem.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
// Async to respect the interface of `load_file_string`.
#[allow(clippy::unused_async)]
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_file_buffer<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(&path)
}

/// Load a file from a server URI. \
/// load_file_string("scene.gltf") -> "http://localhost:3000/scene.gltf"
///
/// # Errors
///
/// Returns an error if the file cannot be donwload or read as a string.
#[cfg(target_arch = "wasm32")]
pub async fn load_file_string<P: AsRef<std::path::Path>>(path: P) -> Result<String, JsValue> {
    let resp = fetch_file(path).await?;

    // Convert this other `Promise` into a rust `Future`.
    let text = JsFuture::from(resp.text()?).await?;
    Ok(text.as_string().unwrap())
}

/// Load a file from the filesystem.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read as a string.
// Async to respect the interface of `load_file_string`.
#[allow(clippy::unused_async)]
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_file_string<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<String, std::io::Error> {
    std::fs::read_to_string(&path)
}
