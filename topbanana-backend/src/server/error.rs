
use rocket::{Request, Catcher, catch, catchers};
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use diesel::result::{DatabaseErrorKind, Error as DieselError};

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
  pub const NOT_FOUND_MESSAGE: &'static str = "Not Found";
  pub const UNKNOWN_DB_ERROR_MESSAGE: &'static str = "An unexpected database error occurred";

  pub fn bad_request(message: &str) -> ApiError {
    ApiError {
      status: Status::BadRequest,
      message: message.to_string(),
    }
  }

  pub fn unauthorized(message: &str) -> ApiError {
    ApiError {
      status: Status::Unauthorized,
      message: message.to_string(),
    }
  }

  pub fn forbidden(message: &str) -> ApiError {
    ApiError {
      status: Status::Forbidden,
      message: message.to_string(),
    }
  }

  pub fn not_found(message: &str) -> ApiError {
    ApiError {
      status: Status::NotFound,
      message: message.to_string(),
    }
  }

  pub fn conflict(message: &str) -> ApiError {
    ApiError {
      status: Status::Conflict,
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

  /// As `ApiError::from` but traets [`DieselError::NotFound`] as an
  /// HTTP 400 rather than HTTP 404. This is suitable to use on
  /// creation requests, where the primary task is not the lookup and
  /// hence failure to lookup is a Bad Request.
  pub fn from_on_create(err: DieselError) -> ApiError {
    if let DieselError::NotFound = err {
      ApiError::bad_request(Self::NOT_FOUND_MESSAGE)
    } else {
      ApiError::from(err)
    }
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

impl From<DieselError> for ApiError {
  fn from(err: DieselError) -> ApiError {
    if let DieselError::NotFound = err {
      ApiError::not_found(Self::NOT_FOUND_MESSAGE)
    } else if let DieselError::DatabaseError(kind, info) = err {
      match kind {
        DatabaseErrorKind::UniqueViolation =>
          ApiError::conflict(&format!("Uniqueness error: {}", info.message())),
        DatabaseErrorKind::ForeignKeyViolation =>
          ApiError::bad_request(&format!("Foreign key violation: {}", info.message())),
        _ =>
          ApiError::internal_server_error(Self::UNKNOWN_DB_ERROR_MESSAGE),
      }
    } else {
      ApiError::internal_server_error(Self::UNKNOWN_DB_ERROR_MESSAGE)
    }
  }
}

/// Extension trait adding [`ServerError`] converters to `Result<T, E>`.
pub trait ApiErrorExt {
  type Output;

  fn map_500_json(self) -> Result<Self::Output, ApiError>;
}

impl<T, E: Display + 'static> ApiErrorExt for Result<T, E> {
  type Output = T;

  fn map_500_json(self) -> Result<Self::Output, ApiError> {
    self.map_err(|err| ApiError::internal_server_error(&err))
  }
}

pub fn catchers() -> Vec<Catcher> {
  catchers![
    bad_request_catcher,
    unauthorized_catcher,
    forbidden_catcher,
  ]
}

#[catch(400)]
pub fn bad_request_catcher(_: &Request) -> ApiError {
  ApiError::bad_request("Bad Request")
}

#[catch(401)]
pub fn unauthorized_catcher(_: &Request) -> ApiError {
  ApiError::unauthorized("Unauthorized")
}

#[catch(403)]
pub fn forbidden_catcher(_: &Request) -> ApiError {
  ApiError::forbidden("Forbidden")
}
