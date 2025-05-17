use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to connect to nyaa")]
    BadGateway(reqwest::Error),
    
    #[error("Unable to read response")]
    ReadFailure,
    
    #[error("Unable to create response")]
    CreateResponseFailure,
    
    #[error("Unable to parse html page as utf-8")]
    BadUtf8,
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self { 
            Self::BadGateway(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message: String = match &self {
            Self::BadGateway(error) => format!("{}: {}", self, error),
            _ => self.to_string(),
        };
        
        (self.status_code(), message).into_response()
    }
}
