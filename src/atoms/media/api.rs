use crate::shell::client;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, File};
use crate::atoms::media::Image;
use crate::atoms::tasks::state::TASKS;


const MULTIPART_THRESHOLD: usize = 5 * 1024 * 1024; // 5MB
const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5MB per part


#[derive(Debug, Serialize)]
pub struct CreateImageRequest {
    pub url: String,
    pub order: Option<i32>,
}

#[derive(Serialize)]
struct InitiateUploadRequest {
    block_id: String,
    file_name: String,
    content_type: String,
    file_size: usize,
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
struct CompleteMultipartRequest {
    block_id: String,
    image_id: String,
    upload_id: String,
    extension: String,
    parts: Vec<CompletedPart>,
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

// POST /projects/{pid}/blocks/{id}/images - create image record
pub async fn create_image(
    project_id: &str,
    block_id: &str,
    url: String,
    order: Option<i32>,
) -> Result<Image, String> {
    let endpoint = format!("/projects/{}/blocks/{}/images", project_id, block_id);
    let request = CreateImageRequest { url, order };
    client::post::<CreateImageRequest, Image>(&endpoint, &request).await
}

// GET /projects/{pid}/blocks/{id}/images - list block images
pub async fn list_block_images( block_id: &str) -> Result<Vec<Image>, String> {
    let endpoint = format!("/blocks/{}/images",  block_id);
    client::get::<Vec<Image>>(&endpoint).await
}

/// Common S3 upload logic (handles single and multipart)
/// Returns (image_id, image_url)
async fn upload_file_to_s3(
    block_id: &str,
    file: File,
) -> Result<(String, String), String> {
    let file_name = file.name();
    let content_type = if file.type_().is_empty() {
        "application/octet-stream".to_string()
    } else {
        file.type_()
    };
    let file_size = file.size() as usize;

    // Step 1: Initiate upload
    let api_url = crate::shell::client::API_BASE_URL;

    let initiate_request = InitiateUploadRequest {
        block_id: block_id.to_string(),
        file_name: file_name.clone(),
        content_type: content_type.clone(),
        file_size,
    };

    // Use new auth API token; fail clearly if not logged in
    let token = crate::api::get_access_token()
        .ok_or_else(|| "No auth token found for upload".to_string())?;

    let response = Request::post(&format!("{}/annotate/upload/initiate", api_url))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&initiate_request)
        .map_err(|e| format!("Failed to serialize initiate request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to initiate upload: {}", e))?;

    if !response.ok() {
        if response.status() == 401 {
            crate::shell::client::handle_unauthorized();
            return Err("Unauthorized - please log in again".to_string());
        }

        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Failed to initiate upload: {}", error_text));
    }

    let initiate_response: InitiateUploadResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse initiate response: {}", e))?;

    // Step 2: Upload file data (S3)
    let image_url = if initiate_response.is_multipart {
        // Multipart upload
        let image_id = upload_multipart(
            file,
            &initiate_response.upload_urls,
            block_id,
            &initiate_response.image_id,
            initiate_response.upload_id.as_ref().unwrap(),
            &initiate_response.extension,
        )
        .await?;

        format!(
            "https://doxle-annotations.s3.amazonaws.com/annotations/blocks/{}/images/{}.{}",
             block_id, image_id, initiate_response.extension
        )
    } else {
        // Single part upload
        upload_single_part(file, &initiate_response.upload_urls[0].upload_url).await?;

        // Call complete endpoint to trigger image processing
        let token = crate::api::get_access_token()
            .ok_or_else(|| "No auth token found for upload".to_string())?;
        let complete_request = CompleteMultipartRequest {
            block_id: block_id.to_string(),
            image_id: initiate_response.image_id.clone(),
            upload_id: String::new(), // Empty for single-part
            extension: initiate_response.extension.clone(),
            parts: vec![], // Empty for single-part
        };

        let response = Request::post(&format!("{}/annotate/upload/complete", api_url))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&complete_request)
            .map_err(|e| format!("Failed to serialize complete request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to complete upload: {}", e))?;

        if !response.ok() {
            if response.status() == 401 {
                crate::shell::client::handle_unauthorized();
                return Err("Unauthorized - please log in again".to_string());
            }
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Failed to complete upload: {}", error_text));
        }

        let complete_response: UploadCompleteResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse complete response: {}", e))?;

        complete_response.url
    };

    Ok((initiate_response.image_id, image_url))
}

/// Upload a file for a Storage Block (no task link)
pub async fn upload_image_for_block(block_id: &str, file: File) -> Result<String, String> {
    // Reuse common S3 logic
    let (image_id, image_url) = upload_file_to_s3(block_id, file).await?;

    // Step 3: Create generic image record in database
    let project_id = "default"; 

    match create_image(project_id, block_id, image_url.clone(), None).await {
        Ok(image) => {
            dioxus::logger::tracing::info!("✅ Image record created: {}", image.image_id);
            Ok(image.image_id)
        }
        Err(e) => {
            dioxus::logger::tracing::error!("❌ Failed to create image record: {}", e);
            Ok(image_id)
        }
    }
}

/// Upload a task-specific file to S3 and create the task image record
pub async fn upload_image_for_task(
    block_id: &str,
    task_id: &str,
    file: File,
) -> Result<String, String> {
    // Reuse common S3 logic
    let (image_id, image_url) = upload_file_to_s3(block_id, file).await?;

    // Step 3: Create TASK image record
    match crate::atoms::tasks::api::api_create_task_image(block_id, task_id, image_url.clone()).await {
        Ok(image) => {
            dioxus::logger::tracing::info!("✅ Task Image record created: {}", image.image_id);
            // Also update TASKS so UI immediately sees the image
            let block_id = block_id.to_string();
            let task_id = task_id.to_string();
            let mut tasks = TASKS.write();
            for task in tasks.iter_mut(){
                if task.task_id == task_id && task.block_id == block_id {
                    task.images.push(image.clone());
                    break; // stop after first match
                }
            } 


            Ok(image.image_id)
        }
        Err(e) => {
            dioxus::logger::tracing::error!("❌ Failed to create task image record: {}", e);
            Ok(image_id)
        }
    }
}

/// Upload file in a single part (< 5MB)
async fn upload_single_part(file: File, upload_url: &str) -> Result<(), String> {
    // Read file as ArrayBuffer
    let array_buffer = read_file_as_array_buffer(&file)
        .await
        .map_err(|e| format!("Failed to read file: {:?}", e))?;

    // Upload to S3 presigned URL
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

/// Upload file in multiple parts (>= 5MB)
async fn upload_multipart(
    file: File,
    upload_urls: &[UploadPart],
    block_id: &str,
    image_id: &str,
    upload_id: &str,
    extension: &str,
) -> Result<String, String> {
    let file_size = file.size() as usize;
    let mut completed_parts = Vec::new();

    // Upload each part
    for (idx, upload_part) in upload_urls.iter().enumerate() {
        let start = idx * CHUNK_SIZE;
        let end = ((idx + 1) * CHUNK_SIZE).min(file_size);

        // Create blob slice for this part
        let blob_part = file
            .slice_with_f64_and_f64(start as f64, end as f64)
            .map_err(|e| format!("Failed to slice file: {:?}", e))?;

        // Read as ArrayBuffer
        let array_buffer = read_blob_as_array_buffer(&blob_part)
            .await
            .map_err(|e| format!("Failed to read part {}: {:?}", upload_part.part_number, e))?;

        // Upload part
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

        // Get ETag from response headers
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

    // Step 3: Complete multipart upload
    let api_url = crate::shell::client::API_BASE_URL;

    let complete_request = CompleteMultipartRequest {
        block_id: block_id.to_string(),
        image_id: image_id.to_string(),
        upload_id: upload_id.to_string(),
        extension: extension.to_string(),
        parts: completed_parts,
    };

    let token = crate::api::get_access_token()
        .ok_or_else(|| "No auth token found for upload".to_string())?;
    let response = Request::post(&format!("{}/annotate/upload/complete", api_url))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&complete_request)
        .map_err(|e| format!("Failed to serialize complete request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to complete multipart upload: {}", e))?;

    if !response.ok() {
        if response.status() == 401 {
                crate::shell::client::handle_unauthorized();
                return Err("Unauthorized - please log in again".to_string());
        }
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!(
            "Failed to complete multipart upload: {}",
            error_text
        ));
    }

    let complete_response: UploadCompleteResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse complete response: {}", e))?;

    Ok(complete_response.image_id)
}

/// Read File as ArrayBuffer using FileReader
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

/// Read Blob as ArrayBuffer using FileReader
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
