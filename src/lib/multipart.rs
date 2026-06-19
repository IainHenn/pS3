use axum::body::{Body, Bytes};
use axum::http::{header, Request};
use futures_util::StreamExt;
use multer::Multipart;

#[derive(Debug)]
pub enum MultipartParseError {
    MissingContentType,
    InvalidBoundary(multer::Error),
    Parse(multer::Error),
}

#[derive(Debug, Clone)]
pub struct FileUpload {
    pub name: String,
    pub mime_type: String,
    pub bytes: Bytes,
}

impl FileUpload {
    pub fn new(name: String, mime_type: String, bytes: Bytes) -> Self {
        Self {
            name,
            mime_type,
            bytes,
        }
    }

    pub fn size(&self) -> i64 {
        self.bytes.len() as i64
    }
}

pub async fn parse_file_upload_from_request(
    request: Request<Body>,
) -> Result<FileUpload, MultipartParseError> {
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .ok_or(MultipartParseError::MissingContentType)?;

    let boundary = multer::parse_boundary(content_type)
        .map_err(MultipartParseError::InvalidBoundary)?;

    let body = request.into_body();
    let stream = body
        .into_data_stream()
        .map(|result| result.map_err(std::io::Error::other));

    let mut multipart = Multipart::new(stream, boundary);
    parse_file_upload(&mut multipart)
        .await
        .map_err(MultipartParseError::Parse)
}

pub async fn parse_file_upload(multipart: &mut Multipart<'_>) -> Result<FileUpload, multer::Error> {
    let mut name = String::new();
    let mut mime_type = String::new();
    let mut bytes = Bytes::new();

    while let Some(field) = multipart.next_field().await? {
        if let Some(file_name) = field.file_name() {
            name = file_name.to_string();
        }

        if let Some(content_type) = field.content_type() {
            mime_type = content_type.to_string();
        }

        bytes = field.bytes().await?;
    }

    Ok(FileUpload::new(name, mime_type, bytes))
}
