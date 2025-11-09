use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode, Request, Headers, Response};

pub async fn set_cloudfront_cookies(api_base: &str, id_token: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;

    // Build request
    let url = format!("{}/auth/cloudfront-cookies", api_base.trim_end_matches('/'));
    let mut init = RequestInit::new();
    init.method("POST");
    init.mode(RequestMode::Cors);
    // IMPORTANT: include credentials so Set-Cookie is accepted cross-origin
    init.credentials(web_sys::RequestCredentials::Include);

    let headers = Headers::new().map_err(|e| format!("headers error: {:?}", e))?;
    headers.append("Authorization", &format!("Bearer {}", id_token)).map_err(|e| format!("header append: {:?}", e))?;
    headers.append("Content-Type", "application/json").map_err(|e| format!("header append: {:?}", e))?;

    init.headers(&headers);

    let request = Request::new_with_str_and_init(&url, &init).map_err(|e| format!("request init: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("fetch error: {:?}", e))?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "bad response".to_string())?;

    if !resp.ok() {
        return Err(format!("cookie request failed: {}", resp.status()));
    }
    Ok(())
}
