use js_sys::{Function, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// WASM browser client for `AgentService.RunAgent` via Connect-ES
/// server-streaming.
///
/// The Connect binary protocol uses a series of length-prefixed protobuf
/// frames over a single HTTP/1.1 POST response body. Each frame is
/// dispatched to `on_event` as a `Uint8Array`.
///
/// Usage:
/// ```js
/// const stream = new AgentStream("https://gateway.example.com", jwt_token);
/// await stream.open('{"agent_id":"...","session_id":"..."}', (frameBytes) => {
///   // decode protobuf frame
/// });
/// ```
#[wasm_bindgen]
pub struct AgentStream {
    gateway_url: String,
    token: String,
}

#[wasm_bindgen]
impl AgentStream {
    /// Create a new `AgentStream` pointing at `gateway_url` with a Bearer
    /// `token` for authentication.
    #[wasm_bindgen(constructor)]
    pub fn new(gateway_url: &str, token: &str) -> Self {
        Self {
            gateway_url: gateway_url.to_owned(),
            token: token.to_owned(),
        }
    }

    /// Open a server-streaming `RunAgent` call.
    ///
    /// `request_json` must be a JSON-encoded `AgentRunRequest`.
    /// `on_event` is called for each raw protobuf frame received
    /// (as a `Uint8Array`).
    ///
    /// Returns a `Promise<void>` that resolves when the stream closes
    /// normally, or rejects on error.
    pub fn open(&self, request_json: &str, on_event: Function) -> Promise {
        let url = format!("{}/flint.v1.AgentService/RunAgent", self.gateway_url);
        let token = self.token.clone();
        let request_json = request_json.to_owned();

        wasm_bindgen_futures::future_to_promise(async move {
            let opts = web_sys::RequestInit::new();
            opts.set_method("POST");
            opts.set_credentials(web_sys::RequestCredentials::SameOrigin);

            let headers = web_sys::Headers::new()
                .map_err(|e| JsValue::from_str(&format!("headers error: {e:?}")))?;
            headers
                .set("Content-Type", "application/connect+json")
                .map_err(|e| JsValue::from_str(&format!("header error: {e:?}")))?;
            headers
                .set("Connect-Protocol-Version", "1")
                .map_err(|e| JsValue::from_str(&format!("header error: {e:?}")))?;
            if !token.is_empty() {
                headers
                    .set("Authorization", &format!("Bearer {token}"))
                    .map_err(|e| JsValue::from_str(&format!("auth header error: {e:?}")))?;
            }
            opts.set_headers(&headers);
            opts.set_body(&JsValue::from_str(&request_json));

            let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
            let request = web_sys::Request::new_with_str_and_init(&url, &opts)
                .map_err(|e| JsValue::from_str(&format!("request error: {e:?}")))?;

            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| JsValue::from_str(&format!("fetch error: {e:?}")))?;

            let resp: web_sys::Response = resp_value
                .dyn_into()
                .map_err(|_| JsValue::from_str("not a Response"))?;

            if !resp.ok() {
                return Err(JsValue::from_str(&format!(
                    "RunAgent failed: HTTP {}",
                    resp.status()
                )));
            }

            // Read Connect-ES length-prefixed frames from the response body.
            // Each frame: 5-byte header (1 flag byte + 4-byte big-endian length)
            // followed by `length` bytes of message data.
            let body = resp
                .body()
                .ok_or_else(|| JsValue::from_str("no response body"))?;
            let reader: web_sys::ReadableStreamDefaultReader = body
                .get_reader()
                .dyn_into()
                .map_err(|_| JsValue::from_str("body is not a ReadableStream"))?;

            let mut buf: Vec<u8> = Vec::new();

            loop {
                let chunk = JsFuture::from(reader.read())
                    .await
                    .map_err(|e| JsValue::from_str(&format!("read error: {e:?}")))?;

                let done = js_sys::Reflect::get(&chunk, &JsValue::from_str("done"))
                    .map_err(|_| JsValue::from_str("missing done field"))?
                    .as_bool()
                    .unwrap_or(true);

                if done {
                    break;
                }

                let value = js_sys::Reflect::get(&chunk, &JsValue::from_str("value"))
                    .map_err(|_| JsValue::from_str("missing value field"))?;

                let uint8 = js_sys::Uint8Array::new(&value);
                buf.extend_from_slice(&uint8.to_vec());

                // Dispatch complete frames.
                while buf.len() >= 5 {
                    let msg_len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
                    if buf.len() < 5 + msg_len {
                        break;
                    }
                    let frame: Vec<u8> = buf.drain(0..5 + msg_len).skip(5).collect();
                    let js_frame = js_sys::Uint8Array::from(frame.as_slice());
                    on_event
                        .call1(&JsValue::NULL, &js_frame)
                        .map_err(|e| JsValue::from_str(&format!("callback error: {e:?}")))?;
                }
            }

            Ok(JsValue::UNDEFINED)
        })
    }
}
