use wasm_bindgen::prelude::*;

/// Publish a JSON payload to a channel via HTTP POST.
///
/// `endpoint` — base URL of the frf-gateway (e.g. `https://api.example.com`)
/// `channel_path` — logical channel path (e.g. `entity/my-room`)
/// `payload` — arbitrary JSON value to publish as the event body
/// `token` — optional JWT Bearer token for authentication
///
/// Returns a `Promise<undefined>` that rejects on network or serialization error.
#[wasm_bindgen]
pub async fn publish(
    endpoint: &str,
    channel_path: &str,
    payload: JsValue,
    token: Option<String>,
) -> Result<(), JsValue> {
    let body = js_sys::JSON::stringify(&payload)
        .map_err(|e| JsValue::from_str(&format!("serialization error: {e:?}")))?;
    let body_str: String = body.into();

    let url = format!("{endpoint}/v1/publish");

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_credentials(web_sys::RequestCredentials::SameOrigin);

    let headers =
        web_sys::Headers::new().map_err(|e| JsValue::from_str(&format!("headers error: {e:?}")))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| JsValue::from_str(&format!("header set error: {e:?}")))?;
    headers
        .set("X-Channel-Path", channel_path)
        .map_err(|e| JsValue::from_str(&format!("header set error: {e:?}")))?;
    if let Some(ref t) = token {
        headers
            .set("Authorization", &format!("Bearer {t}"))
            .map_err(|e| JsValue::from_str(&format!("auth header error: {e:?}")))?;
    }
    opts.set_headers(&headers);
    opts.set_body(&JsValue::from_str(&body_str));

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
            "publish failed: HTTP {}",
            resp.status()
        )));
    }

    Ok(())
}
