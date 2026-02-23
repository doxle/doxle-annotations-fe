use std::cell::Cell;
use std::rc::Rc;

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{Blob, File};

use crate::shell::client::API_BASE_URL;

const CHUNK_SIZE: usize = 50 * 1024 * 1024; // 50MB per part

// ─── Types ───

#[derive(Serialize)]
struct InitiateImportRequest {
    file_name: String,
    file_size: usize,
}

#[derive(Deserialize)]
struct InitiateImportResponse {
    import_id: String,
    s3_key: String,
    upload_url: String,
    upload_id: Option<String>,
    upload_urls: Vec<ImportUploadPart>,
    is_multipart: bool,
}

#[derive(Deserialize)]
struct ImportUploadPart {
    part_number: i32,
    upload_url: String,
}

#[derive(Serialize)]
struct CompleteImportUploadRequest {
    import_id: String,
    s3_key: String,
    upload_id: String,
    parts: Vec<CompletedPart>,
}

#[derive(Serialize, Clone)]
struct CompletedPart {
    part_number: i32,
    etag: String,
}

#[derive(Serialize)]
struct AbortImportUploadRequest {
    s3_key: String,
    upload_id: String,
}

/// Result returned to the UI after upload completes
pub struct ImportUploadResult {
    pub import_id: String,
    pub s3_key: String,
}

/// Result returned from the process endpoint
#[derive(Deserialize)]
pub struct ProcessImportResult {
    pub status: String,
    pub labels_created: usize,
    pub tasks_created: usize,
    pub images_created: usize,
    pub annotations_created: usize,
}

/// Call BE to process the uploaded zip (parse COCO, create tasks/images/annotations)
pub async fn process_import(block_id: &str, import_id: &str, s3_key: &str) -> Result<ProcessImportResult, String> {
    let body = serde_json::json!({
        "import_id": import_id,
        "s3_key": s3_key,
    });

    let response = Request::post(&format!("{}/blocks/{}/import/process", API_BASE_URL, block_id))
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Failed to serialize request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to call process import: {}", e))?;

    if !response.ok() {
        if response.status() == 401 {
            crate::shell::client::handle_unauthorized();
            return Err("Unauthorized - please log in again".to_string());
        }
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Import processing failed: {}", error_text));
    }

    response
        .json::<ProcessImportResult>()
        .await
        .map_err(|e| format!("Failed to parse process response: {}", e))
}

// ─── Batch Import API ───

#[derive(Deserialize)]
pub struct ParseImportResult {
    pub format: String,
    pub total_labels: usize,
    pub total_tasks: usize,
    pub total_images: usize,
    pub total_annotations: usize,
}

#[derive(Deserialize)]
pub struct ProcessBatchResult {
    pub phase: String,
    pub processed: usize,
    pub total: usize,
    pub labels_created: usize,
    pub tasks_created: usize,
    pub images_created: usize,
    pub annotations_created: usize,
}

/// Step 1: Parse zip and create manifest
pub async fn parse_import(block_id: &str, import_id: &str, s3_key: &str) -> Result<ParseImportResult, String> {
    let body = serde_json::json!({
        "import_id": import_id,
        "s3_key": s3_key,
    });

    let response = Request::post(&format!("{}/blocks/{}/import/parse", API_BASE_URL, block_id))
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Failed to serialize: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to call parse import: {}", e))?;

    if !response.ok() {
        if response.status() == 401 {
            crate::shell::client::handle_unauthorized();
            return Err("Unauthorized - please log in again".to_string());
        }
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Parse failed: {}", error_text));
    }

    response.json::<ParseImportResult>().await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Step 2: Process a batch of items for a given phase
pub async fn process_batch(
    block_id: &str,
    s3_key: &str,
    phase: &str,
    offset: usize,
    limit: usize,
) -> Result<ProcessBatchResult, String> {
    let body = serde_json::json!({
        "s3_key": s3_key,
        "phase": phase,
        "offset": offset,
        "limit": limit,
    });

    let response = Request::post(&format!("{}/blocks/{}/import/process-batch", API_BASE_URL, block_id))
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Failed to serialize: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to call process-batch: {}", e))?;

    if !response.ok() {
        if response.status() == 401 {
            crate::shell::client::handle_unauthorized();
            return Err("Unauthorized - please log in again".to_string());
        }
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Batch failed: {}", error_text));
    }

    response.json::<ProcessBatchResult>().await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Step 3: Delete the zip from S3
pub async fn cleanup_import(block_id: &str, s3_key: &str) -> Result<(), String> {
    let body = serde_json::json!({
        "s3_key": s3_key,
    });

    let response = Request::post(&format!("{}/blocks/{}/import/cleanup", API_BASE_URL, block_id))
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Failed to serialize: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to call cleanup: {}", e))?;

    if !response.ok() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Cleanup failed: {}", error_text));
    }

    Ok(())
}

/// Metadata needed to abort an in-progress upload
#[derive(Clone, Default)]
pub struct UploadSession {
    pub block_id: String,
    pub s3_key: String,
    pub upload_id: String,
}

// ─── Public API ───

/// Upload a zip file for block import. Calls initiate → S3 upload → complete.
/// Reports progress via the callback (bytes_uploaded, total_bytes).
/// Checks `cancel_flag` between each chunk; returns Err("cancelled") if set.
/// Writes session info to `session` so the caller can abort if needed.
pub async fn upload_import_zip(
    block_id: &str,
    file: File,
    cancel_flag: Rc<Cell<bool>>,
    session: Rc<Cell<Option<UploadSession>>>,
    mut on_progress: impl FnMut(usize, usize),
) -> Result<ImportUploadResult, String> {
    let file_name = file.name();
    let file_size = file.size() as usize;

    // Step 1: Initiate import (get presigned URLs from BE)
    let initiate_request = InitiateImportRequest {
        file_name,
        file_size,
    };

    let response = Request::post(&format!("{}/blocks/{}/import/initiate", API_BASE_URL, block_id))
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(&initiate_request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to initiate import: {}", e))?;

    if !response.ok() {
        if response.status() == 401 {
            crate::shell::client::handle_unauthorized();
            return Err("Unauthorized - please log in again".to_string());
        }
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Failed to initiate import: {}", error_text));
    }

    let initiate: InitiateImportResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse initiate response: {}", e))?;

    // Store session so caller can abort
    session.set(Some(UploadSession {
        block_id: block_id.to_string(),
        s3_key: initiate.s3_key.clone(),
        upload_id: initiate.upload_id.clone().unwrap_or_default(),
    }));

    // Step 2: Upload to S3
    if initiate.is_multipart {
        on_progress(0, file_size);

        let mut completed_parts = Vec::new();
        let mut bytes_uploaded: usize = 0;

        for (idx, part) in initiate.upload_urls.iter().enumerate() {
            // Check cancellation before each part
            if cancel_flag.get() {
                return Err("cancelled".to_string());
            }

            let start = idx * CHUNK_SIZE;
            let end = ((idx + 1) * CHUNK_SIZE).min(file_size);
            let chunk_len = end - start;

            let blob_part = file
                .slice_with_f64_and_f64(start as f64, end as f64)
                .map_err(|e| format!("Failed to slice file: {:?}", e))?;

            let array_buffer = read_blob_as_array_buffer(&blob_part)
                .await
                .map_err(|e| format!("Failed to read part {}: {:?}", part.part_number, e))?;

            let resp = Request::put(&part.upload_url)
                .body(&array_buffer)
                .map_err(|e| format!("Failed to create upload request: {}", e))?
                .send()
                .await
                .map_err(|e| format!("Failed to upload part {}: {}", part.part_number, e))?;

            if !resp.ok() {
                return Err(format!("Part {} upload failed: {}", part.part_number, resp.status()));
            }

            let etag = resp
                .headers()
                .get("etag")
                .ok_or_else(|| format!("Missing ETag for part {}", part.part_number))?
                .trim_matches('"')
                .to_string();

            completed_parts.push(CompletedPart {
                part_number: part.part_number,
                etag,
            });

            bytes_uploaded += chunk_len;
            on_progress(bytes_uploaded, file_size);
        }

        // Step 3: Complete multipart upload
        let complete_request = CompleteImportUploadRequest {
            import_id: initiate.import_id.clone(),
            s3_key: initiate.s3_key.clone(),
            upload_id: initiate.upload_id.unwrap_or_default(),
            parts: completed_parts,
        };

        let resp = Request::post(&format!("{}/blocks/{}/import/complete", API_BASE_URL, block_id))
            .credentials(web_sys::RequestCredentials::Include)
            .header("Content-Type", "application/json")
            .json(&complete_request)
            .map_err(|e| format!("Failed to serialize complete request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to complete upload: {}", e))?;

        if !resp.ok() {
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Failed to complete upload: {}", error_text));
        }

        session.set(None);
        Ok(ImportUploadResult {
            import_id: initiate.import_id,
            s3_key: initiate.s3_key,
        })
    } else {
        // Single-part upload
        on_progress(0, file_size);

        let array_buffer = read_file_as_array_buffer(&file)
            .await
            .map_err(|e| format!("Failed to read file: {:?}", e))?;

        let resp = Request::put(&initiate.upload_url)
            .header("Content-Type", "application/zip")
            .body(&array_buffer)
            .map_err(|e| format!("Failed to create upload request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to upload file: {}", e))?;

        if !resp.ok() {
            return Err(format!("Upload failed: {}", resp.status()));
        }

        on_progress(file_size, file_size);
        session.set(None);

        Ok(ImportUploadResult {
            import_id: initiate.import_id,
            s3_key: initiate.s3_key,
        })
    }
}

/// Abort an in-progress multipart upload — tells BE to clean up S3 orphaned parts
pub async fn abort_import_upload(sess: &UploadSession) -> Result<(), String> {
    if sess.upload_id.is_empty() {
        return Ok(()); // single-part upload, nothing to abort on S3
    }

    let body = AbortImportUploadRequest {
        s3_key: sess.s3_key.clone(),
        upload_id: sess.upload_id.clone(),
    };

    let resp = Request::post(&format!("{}/blocks/{}/import/abort", API_BASE_URL, sess.block_id))
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Failed to serialize abort request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to abort upload: {}", e))?;

    if !resp.ok() {
        let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Failed to abort upload: {}", error_text));
    }

    Ok(())
}

// ─── File reading helpers ───

async fn read_file_as_array_buffer(file: &File) -> Result<js_sys::ArrayBuffer, wasm_bindgen::JsValue> {
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reader = web_sys::FileReader::new().unwrap();
        let reader_clone = reader.clone();

        let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
            if let Ok(result) = reader_clone.result() {
                resolve.call1(&wasm_bindgen::JsValue::NULL, &result).unwrap();
            }
        }) as Box<dyn FnMut(_)>);

        let onerror = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
            reject
                .call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::from_str("Failed to read file"))
                .unwrap();
        }) as Box<dyn FnMut(_)>);

        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        reader.read_as_array_buffer(file).unwrap();

        onload.forget();
        onerror.forget();
    });

    let result = JsFuture::from(promise).await?;
    Ok(result.dyn_into::<js_sys::ArrayBuffer>()?)
}

async fn read_blob_as_array_buffer(blob: &Blob) -> Result<js_sys::ArrayBuffer, wasm_bindgen::JsValue> {
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reader = web_sys::FileReader::new().unwrap();
        let reader_clone = reader.clone();

        let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
            if let Ok(result) = reader_clone.result() {
                resolve.call1(&wasm_bindgen::JsValue::NULL, &result).unwrap();
            }
        }) as Box<dyn FnMut(_)>);

        let onerror = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
            reject
                .call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::from_str("Failed to read blob"))
                .unwrap();
        }) as Box<dyn FnMut(_)>);

        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        reader.read_as_array_buffer(blob).unwrap();

        onload.forget();
        onerror.forget();
    });

    let result = JsFuture::from(promise).await?;
    Ok(result.dyn_into::<js_sys::ArrayBuffer>()?)
}
