use axum::extract::FromRequestParts;
use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use std::sync::Arc;

use crate::errors::models::AppError;

pub async fn auth(
    req: Request<Body>,
    next: Next,
    token: Arc<String>,
) -> Result<Response, AppError> {
    let (mut parts, body) = req.into_parts();
    let auth_header =
        TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &()).await;

    match auth_header {
        Ok(TypedHeader(Authorization(bearer))) if bearer.token() == token.as_str() => {
            // Reconstruct the request and pass it to the next service
            let req = Request::from_parts(parts, body);
            Ok(next.run(req).await)
        }
        _ => Err(AppError::UnauthorizedError(
            "Invalid token provided.".to_string(),
        )),
    }
}
