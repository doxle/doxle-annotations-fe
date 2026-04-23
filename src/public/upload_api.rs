use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, File};

use crate::core::client::API_BASE_URL;

const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5MB per part

#[derive(Serialize)]
struct InitiateUploadRequest {
    file_name: String,
    content_type: String,
    file_size: usize,
    upload_namespace: String,
}

#[derive(Deserialize)]
struct InitiateUploadResponse {
    image_id: String,
    upload_id: Option<String>,
    upload_urls: Vec<UploadPart>,
    is_multipart: bool,
    extension: String,
}

#[derive(Deserialize)]
struct UploadPart {
    part_number: i32,
    upload_url: String,
}

#[derive(Serialize)]
struct CompleteUploadRequest {
    image_id: String,
    upload_id: String,
    extension: String,
    parts: Vec<CompletedPart>,
    upload_namespace: String,
}

#[derive(Serialize, Clone)]
struct CompletedPart {
    part_number: i32,
    etag: String,
}

#[derive(Deserialize)]
struct UploadCompleteResponse {
    image_id: String,
    url: String,
}

/// Result of a successful upload — includes both id and url for linking
#[derive(Clone, Serialize, Deserialize)]
pub struct UploadedFileInfo {
    pub image_id: String,
    pub file_name: String,
    pub url: String,
    pub content_type: String,
    pub file_size: u64,
}

/// Public upload — no auth headers. Calls /public/upload/initiate then uploads to S3.
pub async fn public_upload_file(file: File) -> Result<UploadedFileInfo, String> {
    let original_name = file.name();
    let original_size = file.size() as u64;
    let file_name = file.name();
    let content_type = if file.type_().is_empty() {
        "application/pdf".to_string()
    } else {
        file.type_()
    };
    let file_size = file.size() as usize;

    // 1. Initiate upload (no auth)
    let initiate_url = format!("{}/public/upload/initiate", API_BASE_URL);
    let initiate_body = InitiateUploadRequest {
        file_name: file_name.clone(),
        content_type: content_type.clone(),
        file_size,
        upload_namespace: "plan".to_string(),
    };

    let resp = Request::post(&initiate_url)
        .header("Content-Type", "application/json")
        .json(&initiate_body)
        .map_err(|e| format!("Failed to serialize initiate request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to initiate upload: {}", e))?;

    if !resp.ok() {
        let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
        return Err(format!("Initiate failed ({}): {}", resp.status(), txt));
    }

    let initiate_response: InitiateUploadResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse initiate response: {}", e))?;

    // 2. Upload to S3
    let complete_response = if initiate_response.is_multipart {
        upload_multipart(
            file,
            &initiate_response.upload_urls,
            &initiate_response.image_id,
            initiate_response.upload_id.as_ref().unwrap(),
            &initiate_response.extension,
        )
        .await?
    } else {
        upload_single_part(file, &initiate_response.upload_urls[0].upload_url).await?;

        // 3. Complete upload (no auth)
        let complete_url = format!("{}/public/upload/complete", API_BASE_URL);
        let complete_body = CompleteUploadRequest {
            image_id: initiate_response.image_id.clone(),
            upload_id: String::new(),
            extension: initiate_response.extension.clone(),
            parts: vec![],
            upload_namespace: "plan".to_string(),
        };

        let resp = Request::post(&complete_url)
            .header("Content-Type", "application/json")
            .json(&complete_body)
            .map_err(|e| format!("Failed to serialize complete request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to complete upload: {}", e))?;

        if !resp.ok() {
            let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            return Err(format!("Complete failed ({}): {}", resp.status(), txt));
        }

        resp.json::<UploadCompleteResponse>()
            .await
            .map_err(|e| format!("Failed to parse complete response: {}", e))?
    };

    Ok(UploadedFileInfo {
        image_id: complete_response.image_id,
        file_name: original_name,
        url: complete_response.url,
        content_type,
        file_size: original_size,
    })
}

// ─── Public Project API ───

#[derive(Serialize)]
struct PublicUploadedFilePayload {
    image_id: String,
    file_name: String,
    url: String,
    content_type: String,
    file_size: u64,
}

#[derive(Serialize)]
struct CreatePublicProjectRequest {
    project_name: String,
    email: String,
    file_ids: Vec<PublicUploadedFilePayload>,
}

#[derive(Deserialize, Clone)]
pub struct PublicProjectBootstrap {
    pub project_id: String,
    pub block_id: String,
    pub project_name: String,
    pub project_email: String,
}

/// Create a public project with uploaded file links
pub async fn create_public_project(
    project_name: &str,
    email: &str,
    files: Vec<UploadedFileInfo>,
) -> Result<PublicProjectBootstrap, String> {
    let url = format!("{}/public/projects", API_BASE_URL);
    let body = CreatePublicProjectRequest {
        project_name: project_name.to_string(),
        email: email.to_string(),
        file_ids: files.into_iter().map(|f| PublicUploadedFilePayload {
            image_id: f.image_id,
            file_name: f.file_name,
            url: f.url,
            content_type: f.content_type,
            file_size: f.file_size,
        }).collect(),
    };

    let resp = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Failed to serialize request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to create project: {}", e))?;

    if !resp.ok() {
        let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
        return Err(format!("Create project failed ({}): {}", resp.status(), txt));
    }

    resp.json::<PublicProjectBootstrap>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

async fn upload_single_part(file: File, upload_url: &str) -> Result<(), String> {
    let array_buffer = read_file_as_array_buffer(&file)
        .await
        .map_err(|e| format!("Failed to read file: {:?}", e))?;

    let response = Request::put(upload_url)
        .header("Content-Type", &file.type_())
        .body(&array_buffer)
        .map_err(|e| format!("Failed to create upload request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to upload file: {}", e))?;

    if !response.ok() {
        return Err(format!("Upload failed with status: {}", response.status()));
    }
    Ok(())
}

async fn upload_multipart(
    file: File,
    upload_urls: &[UploadPart],
    image_id: &str,
    upload_id: &str,
    extension: &str,
) -> Result<UploadCompleteResponse, String> {
    let file_size = file.size() as usize;
    let mut completed_parts = Vec::new();

    for (idx, upload_part) in upload_urls.iter().enumerate() {
        let start = idx * CHUNK_SIZE;
        let end = ((idx + 1) * CHUNK_SIZE).min(file_size);

        let blob_part = file
            .slice_with_f64_and_f64(start as f64, end as f64)
            .map_err(|e| format!("Failed to slice file: {:?}", e))?;

        let array_buffer = read_blob_as_array_buffer(&blob_part)
            .await
            .map_err(|e| format!("Failed to read part {}: {:?}", upload_part.part_number, e))?;

        let response = Request::put(&upload_part.upload_url)
            .body(&array_buffer)
            .map_err(|e| format!("Failed to create part upload request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to upload part {}: {}", upload_part.part_number, e))?;

        if !response.ok() {
            return Err(format!(
                "Part {} upload failed with status: {}",
                upload_part.part_number,
                response.status()
            ));
        }

        let etag = response
            .headers()
            .get("etag")
            .ok_or_else(|| format!("Missing ETag for part {}", upload_part.part_number))?
            .trim_matches('"')
            .to_string();

        completed_parts.push(CompletedPart {
            part_number: upload_part.part_number,
            etag,
        });
    }

    let complete_url = format!("{}/public/upload/complete", API_BASE_URL);
    let complete_body = CompleteUploadRequest {
        image_id: image_id.to_string(),
        upload_id: upload_id.to_string(),
        extension: extension.to_string(),
        parts: completed_parts,
        upload_namespace: "plan".to_string(),
    };

    let resp = Request::post(&complete_url)
        .header("Content-Type", "application/json")
        .json(&complete_body)
        .map_err(|e| format!("Failed to serialize complete request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to complete multipart upload: {}", e))?;

    if !resp.ok() {
        let txt = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
        return Err(format!("Complete failed ({}): {}", resp.status(), txt));
    }

    resp.json::<UploadCompleteResponse>()
        .await
        .map_err(|e| format!("Failed to parse complete response: {}", e))
}

async fn read_file_as_array_buffer(file: &File) -> Result<js_sys::ArrayBuffer, JsValue> {
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reader = web_sys::FileReader::new().unwrap();
        let reader_clone = reader.clone();

        let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
            if let Ok(result) = reader_clone.result() {
                resolve.call1(&JsValue::NULL, &result).unwrap();
            }
        }) as Box<dyn FnMut(_)>);

        let onerror = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
            reject
                .call1(&JsValue::NULL, &JsValue::from_str("Failed to read file"))
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

async fn read_blob_as_array_buffer(blob: &Blob) -> Result<js_sys::ArrayBuffer, JsValue> {
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reader = web_sys::FileReader::new().unwrap();
        let reader_clone = reader.clone();

        let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
            if let Ok(result) = reader_clone.result() {
                resolve.call1(&JsValue::NULL, &result).unwrap();
            }
        }) as Box<dyn FnMut(_)>);

        let onerror = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
            reject
                .call1(&JsValue::NULL, &JsValue::from_str("Failed to read blob"))
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
