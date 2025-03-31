use serde::Serialize;

use super::error::ResponseError;

#[derive(Debug)]
pub enum Body {
    RawText(String),
    RawBinary(Vec<u8>),
    Json(Vec<u8>),
}

impl Body {
    pub fn len(&self) -> usize {
        match self {
            Body::RawText(s) => s.len(),
            Body::RawBinary(b) => b.len(),
            Body::Json(j) => j.len(),
        }
    }
}

pub trait IntoTextBody
where
    Self: Sized,
{
    fn into_body(self) -> Body;
}

pub trait IntoBinaryBody
where
    Self: Sized,
{
    #[allow(unused)]
    fn into_body(self) -> Body;
}

pub trait IntoJsonBody
where
    Self: Serialize + Sized,
{
    #[allow(unused)]
    fn into_body(self) -> Result<Body, ResponseError>;
}

impl IntoTextBody for () {
    fn into_body(self) -> Body {
        Body::RawText(String::new())
    }
}

impl IntoTextBody for String {
    fn into_body(self) -> Body {
        Body::RawText(self)
    }
}

impl IntoTextBody for &str {
    fn into_body(self) -> Body {
        Body::RawText(self.to_string())
    }
}

impl IntoTextBody for &String {
    fn into_body(self) -> Body {
        Body::RawText(self.to_string())
    }
}

impl IntoTextBody for Vec<u8> {
    fn into_body(self) -> Body {
        match String::from_utf8_lossy(&self) {
            std::borrow::Cow::Borrowed(s) => Body::RawText(s.to_string()),
            std::borrow::Cow::Owned(s) => Body::RawText(s),
        }
    }
}

impl IntoTextBody for &[u8] {
    fn into_body(self) -> Body {
        match String::from_utf8_lossy(self) {
            std::borrow::Cow::Borrowed(s) => Body::RawText(s.to_string()),
            std::borrow::Cow::Owned(s) => Body::RawText(s),
        }
    }
}

impl IntoBinaryBody for () {
    fn into_body(self) -> Body {
        Body::RawBinary(vec![])
    }
}

impl IntoBinaryBody for Vec<u8> {
    fn into_body(self) -> Body {
        Body::RawBinary(self)
    }
}

impl IntoBinaryBody for &[u8] {
    fn into_body(self) -> Body {
        Body::RawBinary(self.to_vec())
    }
}

impl<T: Serialize + Sized> IntoJsonBody for T {
    fn into_body(self) -> Result<Body, ResponseError> {
        Ok(Body::Json(serde_json::to_vec(&self)?))
    }
}
