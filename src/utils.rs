#[cfg(target_arch = "wasm32")]
use web_sys::{Request, Response};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::*, JsValue};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
pub async fn fetch_file<P: AsRef<std::path::Path>>(path: P) -> Result<Response, JsValue> {
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
pub async fn load_file_buffer<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<u8>, JsValue> {
    let resp = fetch_file(path).await?;

    // Convert this other `Promise` into a rust `Future`.
    let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
    let u8_array: js_sys::Uint8Array = js_sys::Uint8Array::new(&array_buffer);

    // Send the JSON response back to JS.
    Ok(u8_array.to_vec())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_file_buffer<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(&path)
}

#[cfg(target_arch = "wasm32")]
pub async fn load_file_string<P: AsRef<std::path::Path>>(path: P) -> Result<String, JsValue> {
    let resp = fetch_file(path).await?;

    // Convert this other `Promise` into a rust `Future`.
    let text = JsFuture::from(resp.text()?).await?;
    Ok(text.as_string().unwrap())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_file_string<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<String, std::io::Error> {
    std::fs::read_to_string(&path)
}
