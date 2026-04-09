use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::models::ErrorResponse;

#[derive(Debug)]
pub enum AppError {
	BadRequest(String),
	NotFound(String),
	Internal(String),
}

impl AppError {
	pub fn bad_request(msg: impl Into<String>) -> Self {
		Self::BadRequest(msg.into())
	}

	pub fn not_found(msg: impl Into<String>) -> Self {
		Self::NotFound(msg.into())
	}

	pub fn internal(msg: impl Into<String>) -> Self {
		Self::Internal(msg.into())
	}
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let (status, message) = match self {
			AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
			AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
			AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
		};

		(status, Json(ErrorResponse { error: message })).into_response()
	}
}

impl From<anyhow::Error> for AppError {
	fn from(err: anyhow::Error) -> Self {
		AppError::Internal(err.to_string())
	}
}
