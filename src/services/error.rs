use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};
use validator::ValidationErrors;

use repos::{Error as ReposError, ErrorKind as ReposErrorKind};

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "service error - unauthorized")]
    Unauthorized,
    #[fail(display = "service error - malformed input")]
    MalformedInput,
    #[fail(display = "service error - invalid input, errors: {}", _0)]
    InvalidInput(ValidationErrors),
    #[fail(display = "service error - internal error")]
    Internal,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorSource {
    #[fail(display = "service error source - r2d2")]
    R2D2,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorContext {
    #[fail(display = "service error context - no auth token received")]
    NoAuthToken,
}

derive_error_impls!();

impl From<ReposError> for Error {
    fn from(e: ReposError) -> Error {
        match e.kind() {
            ReposErrorKind::Internal => e.context(ErrorKind::Internal).into(),
        }
    }
}
