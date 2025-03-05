
use rocket::Request;
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiStatus {
  Success,
  Error,
}

/// Rocket responder which responds using a JSON-like object
/// indicating what went wrong.
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct ApiError {
  status: Status,
  message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorPayload {
  status: ApiStatus,
  reason: String,
}

impl ApiError {
  pub fn bad_request(message: &str) -> ApiError {
    ApiError {
      status: Status::BadRequest,
      message: message.to_string(),
    }
  }

  pub fn forbidden(message: &str) -> ApiError {
    ApiError {
      status: Status::Forbidden,
      message: message.to_string(),
    }
  }

  pub fn internal_server_error(message: &str) -> ApiError {
    ApiError {
      status: Status::InternalServerError,
      message: message.to_string(),
    }
  }

  pub fn status(&self) -> Status {
    self.status
  }

  pub fn message(&self) -> &str {
    &self.message
  }
}

impl ErrorPayload {
  pub fn new(message: String) -> ErrorPayload {
    ErrorPayload {
      status: ApiStatus::Error,
      reason: message,
    }
  }
}

impl<'r> Responder<'r, 'static> for ApiError {
  fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
    let payload = ErrorPayload::new(self.message);
    (self.status, Json(payload)).respond_to(req)
  }
}
