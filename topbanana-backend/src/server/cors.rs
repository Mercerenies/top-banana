
//! Custom responders for modifying CORS headers.

use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::Request;

/// Wrapper for adding wildcard CORS headers.
#[derive(Debug, Clone)]
pub struct WithWildcardCors<T>(pub T);

impl<'r, T: Responder<'r, 'static>> Responder<'r, 'static> for WithWildcardCors<T> {
  fn respond_to(self, req: &'r Request<'_>) -> Result<Response<'static>, Status> {
    let mut response = self.0.respond_to(req)?;
    response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
    response.set_header(Header::new("Access-Control-Allow-Methods", "GET, POST, OPTIONS"));
    response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
    Ok(response)
  }
}
