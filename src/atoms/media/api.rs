use crate::atoms::media::{Image, MarkupRect};
use crate::atoms::tasks::state::TASKS;
use crate::shell::client;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, File};

const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5MB per part

#[derive(Copy, Clone)]
enum UploadNamespace {
    Annotation,
    File,
}

impl UploadNamespace {
    fn as_str(&self) -> &'static str {
        match self {
            UploadNamespace::Annotation => "annotation",
            UploadNamespace::File => "file",
        }
    }
}

#[derive(Serialize)]
struct InitiateUploadRequest {
    block_id: String,
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
struct CompleteMultipartRequest {
    block_id: String,
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

#[derive(Debug, Serialize)]
struct CreateBlockMediaRequest {
    image_id: String,
    image_name: String,
    url: String,
    media_type: String,
}

#[derive(Debug, Serialize)]
struct UpdateImageMarkupRequest {
    markup_rects: Vec<MarkupRect>,
}

/// GET /images/{image_id}?block_id={block_id}
pub async fn api_get_image(block_id: &str, image_id: &str) -> Result<Image, String> {
    let endpoint = format!("/images/{}?block_id={}", image_id, block_id);
    client::get::<Image>(&endpoint).await
}

/// PATCH /images/{image_id}?block_id={block_id}
pub async fn api_update_image_markup(
    project_id: &str,
    block_id: &str,
    image_id: &str,
    markup_rects: Vec<MarkupRect>,
) -> Result<Image, String> {
    let endpoint = format!(
        "/projects/{}/blocks/{}/media/{}/markup",
        project_id, block_id, image_id
    );
    let body = UpdateImageMarkupRequest { markup_rects };
    client::patch::<UpdateImageMarkupRequest, Image>(&endpoint, &body).await
}

/// DELETE /images/{image_id}?block_id={block_id}&project_id={project_id}
pub async fn api_delete_image(project_id: &str, block_id: &str, image_id: &str) -> Result<(), String> {
    let endpoint = format!("/images/{}?block_id={}&project_id={}", image_id, block_id, project_id);
    client::delete(&endpoint).await
}

/// GET /projects/{pid}/blocks/{block_id}/media
pub async fn list_block_media(project_id: &str, block_id: &str) -> Result<Vec<Image>, String> {
    let endpoint = format!("/projects/{}/blocks/{}/media", project_id, block_id);
    client::get::<Vec<Image>>(&endpoint).await
}

/// POST /projects/{pid}/blocks/{block_id}/media
pub async fn create_block_media(
    project_id: &str,
    block_id: &str,
    image_id: String,
    image_name: String,
    url: String,
    media_type: String,
) -> Result<Image, String> {
    let endpoint = format!("/projects/{}/blocks/{}/media", project_id, block_id);
    let body = CreateBlockMediaRequest {
        image_id,
        image_name,
        url,
        media_type,
    };
    client::post::<CreateBlockMediaRequest, Image>(&endpoint, &body).await
}

/// Upload a file for a File block (no task link).
pub async fn upload_image_for_block(project_id: &str, block_id: &str, file: File) -> Result<String, String> {
    let media_type = detect_media_type(&file);
    let (image_id, image_name, image_url) =
        upload_file_to_s3(block_id, file, UploadNamespace::File).await?;

    match create_block_media(
        project_id,
        block_id,
        image_id.clone(),
        image_name,
        image_url,
        media_type,
    )
    .await
    {
        Ok(image) => Ok(image.image_id),
        Err(e) => {
            dioxus::logger::tracing::error!("❌ Failed to create block media record: {}", e);
            Ok(image_id)
        }
    }
}

/// Upload a task-specific file to S3 and create the task image record.
pub async fn upload_image_for_task(
    project_id: &str,
    block_id: &str,
    task_id: &str,
    file: File,
) -> Result<String, String> {
    let (image_id, image_name, image_url) =
        upload_file_to_s3(block_id, file, UploadNamespace::Annotation).await?;

    match crate::atoms::tasks::api::api_create_task_image(
        project_id,
        block_id,
        task_id,
        image_id.clone(),
        image_name,
        image_url,
    )
    .await
    {
        Ok(image) => {
            let block_id = block_id.to_string();
            let task_id = task_id.to_string();
            let mut tasks = TASKS.write();
            for task in tasks.iter_mut() {
                if task.task_id == task_id && task.block_id == block_id {
                    task.images.push(image.clone());
                    break;
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

/// Common S3 upload logic (handles single and multipart).
/// Returns (image_id, image_name, image_url).
async fn upload_file_to_s3(
    block_id: &str,
    file: File,
    upload_namespace: UploadNamespace,
) -> Result<(String, String, String), String> {
    let file_name = file.name();
    let content_type = if file.type_().is_empty() {
        "application/octet-stream".to_string()
    } else {
        file.type_()
    };
    let file_size = file.size() as usize;

    let api_url = crate::shell::client::API_BASE_URL;

    let initiate_request = InitiateUploadRequest {
        block_id: block_id.to_string(),
        file_name: file_name.clone(),
        content_type: content_type.clone(),
        file_size,
        upload_namespace: upload_namespace.as_str().to_string(),
    };

    let response = Request::post(&format!("{}/media/upload/initiate", api_url))
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
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

    let complete_response = if initiate_response.is_multipart {
        upload_multipart(
            file,
            &initiate_response.upload_urls,
            block_id,
            &initiate_response.image_id,
            initiate_response.upload_id.as_ref().unwrap(),
            &initiate_response.extension,
            upload_namespace.as_str(),
        )
        .await?
    } else {
        upload_single_part(file, &initiate_response.upload_urls[0].upload_url).await?;

        let complete_request = CompleteMultipartRequest {
            block_id: block_id.to_string(),
            image_id: initiate_response.image_id.clone(),
            upload_id: String::new(),
            extension: initiate_response.extension.clone(),
            parts: vec![],
            upload_namespace: upload_namespace.as_str().to_string(),
        };

        let response = Request::post(&format!("{}/media/upload/complete", api_url))
            .credentials(web_sys::RequestCredentials::Include)
            .header("Content-Type", "application/json")
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

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse complete response: {}", e))?
    };

    Ok((complete_response.image_id, file_name, complete_response.url))
}

/// Upload file in a single part.
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

/// Upload file in multiple parts.
async fn upload_multipart(
    file: File,
    upload_urls: &[UploadPart],
    block_id: &str,
    image_id: &str,
    upload_id: &str,
    extension: &str,
    upload_namespace: &str,
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

    let api_url = crate::shell::client::API_BASE_URL;

    let complete_request = CompleteMultipartRequest {
        block_id: block_id.to_string(),
        image_id: image_id.to_string(),
        upload_id: upload_id.to_string(),
        extension: extension.to_string(),
        parts: completed_parts,
        upload_namespace: upload_namespace.to_string(),
    };

    let response = Request::post(&format!("{}/media/upload/complete", api_url))
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
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
        return Err(format!("Failed to complete multipart upload: {}", error_text));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse complete response: {}", e))
}

fn detect_media_type(file: &File) -> String {
    let mime = file.type_();
    if mime.starts_with("video/") {
        "video".to_string()
    } else if mime.starts_with("image/") {
        "image".to_string()
    } else {
        "file".to_string()
    }
}

/// Read File as ArrayBuffer using FileReader.
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

/// Read Blob as ArrayBuffer using FileReader.
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
