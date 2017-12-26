pub mod session;

use reqwest::{Error, StatusCode};

pub use self::session::Session;

/// An error that might occur during a reddit request
#[derive(Debug)]
pub enum RedditError
{
    /// Bad client ID or secret
    Unauthorized,
    /// Bad username or password
    BadCredentials,
    /// Another status that isn't 200 OK
    OtherStatus(StatusCode),
    /// Another error
    OtherError(Error),
}

impl From<Error> for RedditError
{
    fn from(err: Error) -> RedditError
    {
        RedditError::OtherError(err)
    }
}
