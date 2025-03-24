use std::fmt::Display;

use super::error::ResponseError;

#[derive(Debug)]
#[repr(u16)]
pub enum HttpStatus {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    NotFound = 404,
}

impl Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpStatus::Ok => write!(f, "200 OK"),
            HttpStatus::Created => write!(f, "201 Created"),
            HttpStatus::BadRequest => write!(f, "400 Bad Request"),
            HttpStatus::NotFound => write!(f, "404 Not Found"),
        }
    }
}

impl TryFrom<u16> for HttpStatus {
    type Error = ResponseError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            200 => Ok(HttpStatus::Ok),
            201 => Ok(HttpStatus::Created),
            400 => Ok(HttpStatus::BadRequest),
            404 => Ok(HttpStatus::NotFound),
            _ => Err(ResponseError::InvalidStatusCode(value)),
        }
    }
}
