#[macro_use]
extern crate log;

use actix_web::{
    error::{BlockingError, ResponseError},
    Error as ActixError, HttpResponse,
};
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error as DBError};
use r2d2::Error as PoolError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, PartialEq)]
pub enum Error {
    BadRequest(String),
    InternalServerError(String),
    Unauthorized,
    Forbidden,
    NotFound(String),
    PoolError(String),
    BlockingError(String),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::BadRequest(error) => {
                HttpResponse::BadRequest().json::<ErrorResponse>(error.into())
            }
            Error::NotFound(message) => {
                HttpResponse::NotFound().json::<ErrorResponse>(message.into())
            }
            Error::Forbidden => HttpResponse::Forbidden().json::<ErrorResponse>("Forbidden".into()),
            _ => {
                error!("Internal server error: {:?}", self);
                HttpResponse::InternalServerError()
                    .json::<ErrorResponse>("Internal Server Error".into())
            }
        }
    }
}
// User-friendly error messages
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub errors: Vec<String>,
}

impl From<&str> for ErrorResponse {
    fn from(error: &str) -> Self {
        ErrorResponse {
            errors: vec![error.into()],
        }
    }
}

impl From<&String> for ErrorResponse {
    fn from(error: &String) -> Self {
        ErrorResponse {
            errors: vec![error.into()],
        }
    }
}

impl From<Vec<String>> for ErrorResponse {
    fn from(error: Vec<String>) -> Self {
        ErrorResponse { errors: error }
    }
}

// Convert DBErrors to our Error type
impl From<DBError> for Error {
    fn from(error: DBError) -> Error {
        // Right now we just care about UniqueViolation from diesel
        // But this would be helpful to easily map errors as our app grows
        match error {
            DBError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return Error::BadRequest(message);
                }
                Error::InternalServerError("Unknown database error".into())
            }
            DBError::NotFound => Error::NotFound("Record not found".into()),
            _ => Error::InternalServerError("Unknown database error".into()),
        }
    }
}

// Convert PoolError to our Error type
impl From<PoolError> for Error {
    fn from(error: PoolError) -> Error {
        Error::PoolError(error.to_string())
    }
}

impl From<BlockingError<Error>> for Error {
    fn from(error: BlockingError<Error>) -> Error {
        match error {
            BlockingError::Error(error) => error,
            BlockingError::Canceled => Error::BlockingError("Thread blocking error".into()),
        }
    }
}

impl From<ActixError> for Error {
    fn from(error: ActixError) -> Error {
        Error::InternalServerError(error.to_string())
    }
}
