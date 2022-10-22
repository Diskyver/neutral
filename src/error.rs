use http::StatusCode;
use tokio::time::error::Elapsed;

#[derive(Debug)]
pub struct NeutrinoError {
    pub status_code: StatusCode,
    pub error: String,
}

#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
    Json(serde_json::Error),
    Timeout(Elapsed),
    Neutrino(NeutrinoError),
    InvalidUri(http::uri::InvalidUri),
    Http(http::Error),
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Self::Hyper(err)
    }
}

impl From<NeutrinoError> for Error {
    fn from(err: NeutrinoError) -> Self {
        Self::Neutrino(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<Elapsed> for Error {
    fn from(err: Elapsed) -> Self {
        Self::Timeout(err)
    }
}

impl From<http::uri::InvalidUri> for Error {
    fn from(err: http::uri::InvalidUri) -> Self {
        Self::InvalidUri(err)
    }
}

impl From<http::Error> for Error {
    fn from(err: http::Error) -> Self {
        Self::Http(err)
    }
}
