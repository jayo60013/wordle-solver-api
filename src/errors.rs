use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    BadRequest {
        detail: String,
        instance: Option<String>,
    },
    RateLimited {
        detail: String,
        instance: Option<String>,
    },
}

#[derive(Serialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: &'static str,
    pub title: &'static str,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl ApiError {
    pub fn bad_request(detail: impl Into<String>, instance: impl Into<String>) -> Self {
        Self::BadRequest {
            detail: detail.into(),
            instance: Some(instance.into()),
        }
    }

    pub fn rate_limited(detail: impl Into<String>, instance: impl Into<String>) -> Self {
        Self::RateLimited {
            detail: detail.into(),
            instance: Some(instance.into()),
        }
    }

    fn to_problem_details(&self) -> ProblemDetails {
        match self {
            Self::BadRequest { detail, instance } => ProblemDetails {
                problem_type: "urn:wordle-solver:problem:bad-request",
                title: "Bad Request",
                status: StatusCode::BAD_REQUEST.as_u16(),
                detail: Some(detail.clone()),
                instance: instance.clone(),
            },
            Self::RateLimited { detail, instance } => ProblemDetails {
                problem_type: "urn:wordle-solver:problem:rate-limit-exceeded",
                title: "Too Many Requests",
                status: StatusCode::TOO_MANY_REQUESTS.as_u16(),
                detail: Some(detail.clone()),
                instance: instance.clone(),
            },
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest { detail, .. }
            | Self::RateLimited { detail, .. } => write!(f, "{detail}"),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .content_type("application/problem+json")
            .json(self.to_problem_details())
    }
}
