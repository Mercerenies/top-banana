
//! Helpers for verifying request UUID and digital signature
//! information.

mod hasher;

pub use hasher::{RequestSigningHasher, SecurityLevel, Sha256Hasher, Sha1Hasher};

use crate::db::{schema, models};
use crate::server::error::ApiError;

use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use uuid::Uuid;
use chrono::{NaiveDateTime, TimeDelta};
use chrono::naive::serde::ts_seconds;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncPgConnection};
use log::{debug, warn};

use std::str::{from_utf8, Utf8Error, FromStr};

/// A payload for a request made from a relevant video game client.
///
/// Payloads of this form consist of two base64url-encoded strings,
/// separated by a dot. The first string is the actual payload, and
/// the second is the digital signature.
///
/// Existence of this structure does NOT guarantee that the signature
/// has been verified. It is possible for this structure to contain
/// unverified (and potentially invalid) signatures.
#[derive(Debug, Clone)]
pub struct GameRequestPayload {
  payload_base64: String,
  signature_base64: String,
}

/// The body of a game request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRequestBody<T> {
  pub game_uuid: Uuid,
  pub request_uuid: Uuid,
  #[serde(with = "ts_seconds")]
  pub request_timestamp: NaiveDateTime,
  pub algo: RequestAlgorithm,
  #[serde(flatten)]
  pub body: T,
}

/// Chosen algorithm for a game request.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum RequestAlgorithm {
  Sha1,
  Sha256,
}

#[derive(Debug, Clone, Error)]
#[error("Invalid GameRequestPayload")]
pub struct GameRequestPayloadFromStrError {
  _priv: (),
}

#[derive(Debug, Clone, Error)]
#[error("Invalid request signature")]
pub struct VerificationError {
  _priv: (),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeserializeError {
  #[error("{0}")]
  JsonError(#[from] serde_json::Error),
  #[error("{0}")]
  Base64Error(#[from] base64::DecodeError),
  #[error("{0}")]
  Utf8Error(#[from] Utf8Error),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RequestBodyVerifyError {
  #[error("{0}")]
  DeserializeError(#[from] DeserializeError),
  #[error("{0}")]
  DieselError(#[from] diesel::result::Error),
  #[error("No such game")]
  NoSuchGame,
  #[error("{0}")]
  VerificationError(#[from] VerificationError),
  #[error("Request timestamp is not current")]
  BadRequestTimestamp,
  #[error("Request has already been seen")]
  RequestAlreadySeen,
  #[error("Security level not attained")]
  SecurityLevelNotAttained,
}

impl GameRequestPayload {
  pub fn new(payload_base64: String, signature_base64: String) -> Self {
    Self {
      payload_base64,
      signature_base64,
    }
  }

  pub fn verify<H>(&self, secret_key: &str, hasher: &H) -> Result<(), VerificationError>
  where H: RequestSigningHasher + ?Sized {
    let full_payload = format!("{}.{}", self.payload_base64, secret_key);
    let expected_signature = hasher.apply_hash(&full_payload);
    let given_signature = URL_SAFE.decode(self.signature_base64.as_bytes()).map_err(|_| VerificationError { _priv: () })?;
    if expected_signature.as_ref() != given_signature.as_slice() {
      return Err(VerificationError { _priv: () });
    }
    Ok(())
  }

  pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T, DeserializeError> {
    let payload = URL_SAFE.decode(&self.payload_base64)?;
    let payload = serde_json::from_str(from_utf8(&payload)?)?;
    Ok(payload)
  }
}

impl<T> GameRequestBody<T> {
  /// Amount of time allowed between the system clock and a request's timestamp.
  pub const TIME_SKEW: TimeDelta = TimeDelta::days(2);

  pub async fn full_verify_at_time(payload: &GameRequestPayload, db: &mut AsyncPgConnection, now: NaiveDateTime) -> Result<Self, RequestBodyVerifyError>
  where T: DeserializeOwned {
    debug!("Verifying payload {:?}", payload);
    let body = payload.deserialize::<Self>()?;
    let hasher = body.algo.into_hasher();
    let (secret_key, security_level) = schema::games::table
      .filter(schema::games::game_uuid.eq(body.game_uuid))
      .select((schema::games::game_secret_key, schema::games::security_level))
      .first::<(String, i32)>(db)
      .await
      .optional()?
      .ok_or(RequestBodyVerifyError::NoSuchGame)?;

    debug!("Found game with uuid {}, security level is {}", body.game_uuid, security_level);

    // Verify that the appropriate security level is being used.
    if i32::from(hasher.security_level()) < security_level {
      warn!("Got a request using security level {} but expected at least {}", i32::from(hasher.security_level()), security_level);
      return Err(RequestBodyVerifyError::SecurityLevelNotAttained);
    }

    // Verify the signing key.
    payload.verify(&secret_key, &*hasher).map_err(|err| {
      warn!("Got bad signing key for game {}", body.game_uuid);
      err
    })?;

    // Verify the date.
    let time_diff = now - body.request_timestamp;
    if time_diff.abs() > Self::TIME_SKEW {
      warn!("Got outdated request timestamp for game {} ({:?})", body.game_uuid, body.request_timestamp);
      return Err(RequestBodyVerifyError::BadRequestTimestamp);
    }

    // Verify that the request UUID has not been seen before.
    let subquery = schema::historical_requests::table
      .filter(schema::historical_requests::request_uuid.eq(&body.request_uuid));
    if diesel::select(diesel::dsl::exists(subquery)).get_result::<bool>(db).await? {
      warn!("Got repeated request with uuid {}", body.request_uuid);
      return Err(RequestBodyVerifyError::RequestAlreadySeen);
    }

    // Everything is good; insert the request UUID into the historical
    // requests table for later.
    let new_row = models::NewHistoricalRequest { request_uuid: body.request_uuid };
    diesel::insert_into(schema::historical_requests::table)
      .values(&new_row)
      .execute(db)
      .await?;

    Ok(body)
  }

  pub async fn full_verify(payload: &GameRequestPayload, db: &mut AsyncPgConnection) -> Result<Self, RequestBodyVerifyError>
  where T: DeserializeOwned {
    let now = chrono::Utc::now().naive_utc();
    Self::full_verify_at_time(payload, db, now).await
  }
}

impl RequestAlgorithm {
  pub fn into_hasher(self) -> Box<dyn RequestSigningHasher + Send + Sync + 'static> {
    match self {
      RequestAlgorithm::Sha1 => Box::new(Sha1Hasher),
      RequestAlgorithm::Sha256 => Box::new(Sha256Hasher),
    }
  }
}

impl FromStr for GameRequestPayload {
  type Err = GameRequestPayloadFromStrError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let Some((payload_base64, signature_base64)) = s.split_once('.') else {
      return Err(GameRequestPayloadFromStrError { _priv: () });
    };
    Ok(GameRequestPayload::new(payload_base64.to_string(), signature_base64.to_string()))
  }
}

impl From<RequestBodyVerifyError> for ApiError {
  fn from(e: RequestBodyVerifyError) -> Self {
    match e {
      RequestBodyVerifyError::DeserializeError(_) => ApiError::bad_request(),
      RequestBodyVerifyError::DieselError(e) => e.into(),
      RequestBodyVerifyError::VerificationError(_) => ApiError::forbidden(),
      RequestBodyVerifyError::BadRequestTimestamp => ApiError::forbidden(),
      RequestBodyVerifyError::RequestAlreadySeen => ApiError::forbidden(),
      RequestBodyVerifyError::NoSuchGame => ApiError::not_found().with_message("No such game"),
      RequestBodyVerifyError::SecurityLevelNotAttained => ApiError::forbidden().with_message("Invalid low-security algorithm"),
    }
  }
}
