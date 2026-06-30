use js_sys::Function;
use wasm_bindgen::prelude::*;

/// Subscribe to an entity stream via WebSocket.
///
/// Opens a WebSocket connection to `{endpoint}/ws/v1/subscribe` and calls
/// `callback(eventJson)` for each inbound message frame. The callback receives
/// a JSON string representation of the `EventEnvelope`.
///
/// `token` — optional JWT Bearer token appended as a `?token=...` query param.
///
/// Returns a `Promise` that resolves to the `WebSocket` handle (so the caller
/// can close it later), and rejects if the connection cannot be established.
#[wasm_bindgen]
pub async fn subscribe(
    endpoint: &str,
    channel_path: &str,
    callback: Function,
    token: Option<String>,
) -> Result<JsValue, JsValue> {
    let ws_url = endpoint
        .replacen("https://", "wss://", 1)
        .replacen("http://", "ws://", 1);

    let ws_url = if let Some(ref t) = token {
        format!("{ws_url}/ws/v1/subscribe?channel={channel_path}&token={t}")
    } else {
        format!("{ws_url}/ws/v1/subscribe?channel={channel_path}")
    };

    let ws = web_sys::WebSocket::new(&ws_url)
        .map_err(|e| JsValue::from_str(&format!("WebSocket open error: {e:?}")))?;

    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    // Wire onmessage → callback.
    let cb = callback.clone();
    let onmessage =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |evt: web_sys::MessageEvent| {
            let data = evt.data();
            let _ = cb.call1(&JsValue::NULL, &data);
        });
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    // Return the WebSocket handle so the caller can close it later.
    Ok(ws.into())
}
