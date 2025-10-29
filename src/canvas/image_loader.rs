use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestCredentials, Response, Blob, Url};

/// Load an image with credentials/cookies and return a blob URL that can be used in an img tag
pub async fn load_authenticated_image(url: &str) -> Result<String, String> {
    let window = web_sys::window().ok_or("No window available")?;
    
    // Add cache-busting parameter to avoid browser using cached 403
    let timestamp = js_sys::Date::now() as u64;
    let cache_bust_url = if url.contains('?') {
        format!("{}&cf_bust={}", url, timestamp)
    } else {
        format!("{}?cf_bust={}", url, timestamp)
    };
    
    // Create request with credentials
    let mut init = RequestInit::new();
    init.method("GET");
    init.mode(web_sys::RequestMode::Cors); // Enable CORS
    init.credentials(RequestCredentials::Include); // Include cookies
    
    let request = Request::new_with_str_and_init(&cache_bust_url, &init)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    
    // Fetch the image
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;
    
    let response: Response = resp_value
        .dyn_into()
        .map_err(|_| "Invalid response type")?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    // Get blob from response
    let blob_promise = response.blob()
        .map_err(|e| format!("Failed to get blob: {:?}", e))?;
    
    let blob_value = JsFuture::from(blob_promise)
        .await
        .map_err(|e| format!("Blob conversion error: {:?}", e))?;
    
    let blob: Blob = blob_value
        .dyn_into()
        .map_err(|_| "Invalid blob type")?;
    
    // Create object URL
    let blob_url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Failed to create blob URL: {:?}", e))?;
    
    Ok(blob_url)
}

/// Revoke a blob URL to free memory
pub fn revoke_blob_url(url: &str) {
    Url::revoke_object_url(url).ok();
}