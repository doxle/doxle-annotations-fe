use crate::api::get_api_url;
use crate::api::{auth_api, images_api};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, File};

const MULTIPART_THRESHOLD: usize = 5 * 1024 * 1024; // 5MB
const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5MB per part

#[derive(Serialize)]
struct InitiateUploadRequest {
    project_id: String,
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
    project_id: String,
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

/// Upload a file to S3 (handles both single and multipart uploads) and create image record
pub async fn upload_file(project_id: &str, block_id: &str, file: File) -> Result<String, String> {
    let file_name = file.name();
    let content_type = if file.type_().is_empty() {
        "application/octet-stream".to_string()
    } else {
        file.type_()
    };
    let file_size = file.size() as usize;

    // Step 1: Initiate upload
    let api_url = get_api_url();
    let initiate_request = InitiateUploadRequest {
        project_id: project_id.to_string(),
        block_id: block_id.to_string(),
        file_name: file_name.clone(),
        content_type: content_type.clone(),
        file_size,
    };

    let token = auth_api::get_token().ok_or("No auth token found")?;

    let response = Request::post(&format!("{}/annotate/upload/initiate", api_url))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&initiate_request)
        .map_err(|e| format!("Failed to serialize initiate request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to initiate upload: {}", e))?;

    if !response.ok() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Failed to initiate upload: {}", error_text));
    }

    let initiate_response: InitiateUploadResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse initiate response: {}", e))?;

    // Step 2: Upload file data
    let image_url = if initiate_response.is_multipart {
        // Multipart upload
        let image_id = upload_multipart(
            file,
            &initiate_response.upload_urls,
            project_id,
            block_id,
            &initiate_response.image_id,
            initiate_response.upload_id.as_ref().unwrap(),
            &initiate_response.extension,
        )
        .await?;

        // Generate URL for multipart upload
        format!(
            "https://doxle-annotations.s3.amazonaws.com/projects/{}/blocks/{}/{}.{}",
            project_id, block_id, image_id, initiate_response.extension
        )
    } else {
        // Single part upload
        upload_single_part(file, &initiate_response.upload_urls[0].upload_url).await?;

        // Call complete endpoint to trigger image processing
        let token = auth_api::get_token().ok_or("No auth token found")?;
        let complete_request = CompleteMultipartRequest {
            project_id: project_id.to_string(),
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
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Failed to complete upload: {}", error_text));
        }

        let complete_response: UploadCompleteResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse complete response: {}", e))?;

        complete_response.url
    };

    // Step 3: Create image record in database
    match images_api::create_image(project_id, block_id, image_url.clone(), None).await {
        Ok(image) => {
            tracing::info!("✅ Image record created: {}", image.image_id);
            Ok(image.image_id)
        }
        Err(e) => {
            tracing::error!("❌ Failed to create image record: {}", e);
            // Still return success if S3 upload worked
            Ok(initiate_response.image_id)
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
    project_id: &str,
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
    let api_url = get_api_url();

    let complete_request = CompleteMultipartRequest {
        project_id: project_id.to_string(),
        block_id: block_id.to_string(),
        image_id: image_id.to_string(),
        upload_id: upload_id.to_string(),
        extension: extension.to_string(),
        parts: completed_parts,
    };

    let token = auth_api::get_token().ok_or("No auth token found")?;

    let response = Request::post(&format!("{}/annotate/upload/complete", api_url))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&complete_request)
        .map_err(|e| format!("Failed to serialize complete request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to complete multipart upload: {}", e))?;

    if !response.ok() {
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
