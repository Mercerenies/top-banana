
use rocket::{Request, Catcher, catch, catchers};
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiStatus {
  Success,
  Error,
}

#[derive(Debug, Clone, Responder)]
pub struct ApiSuccessResponse<T> {
  json: Json<ApiSuccessResponseBody<T>>,
}

#[derive(Debug, Clone, Serialize)]
struct ApiSuccessResponseBody<T> {
  status: ApiStatus,
  #[serde(flatten)]
  body: T,
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

impl<T: Serialize> ApiSuccessResponse<T> {
  pub fn new(body: T) -> ApiSuccessResponse<T> {
    let body = ApiSuccessResponseBody {
      status: ApiStatus::Success,
      body
    };
    ApiSuccessResponse {
      json: Json(body),
    }
  }
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

  /// A 500 Internal Server Error.
  ///
  /// This method takes [`Display`] rather than `str`, as we
  /// frequently pass error-like things to it. We can't take
  /// [`Error`](std::error::Error) since `anyhow` doesn't implement
  /// that.
  pub fn internal_server_error(message: impl Display) -> ApiError {
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

pub fn catchers() -> Vec<Catcher> {
  catchers![bad_request_catcher]
}

#[catch(400)]
pub fn bad_request_catcher(_: &Request) -> ApiError {
  ApiError::bad_request("Bad Request")
}
