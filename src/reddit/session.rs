use reqwest::{blocking::Client, StatusCode};
use reqwest::header::{HeaderName, HeaderValue, HeaderMap};
use reqwest::header::USER_AGENT;

use std::time::{Instant, Duration};

use super::RedditError;

/// A reddit session
///
/// This structure handles reddit authorization and ratelimiting.
/// To make sure a request will be handled properly, call
/// `prepare` before the request, and `update` after it.
#[derive(Debug)]
pub struct Session
{
    id: String,
    secret: String,
    user: String,
    pass: String,

    token: Option<String>,
    expires: Instant,

    remain: u32,
    reset: Instant,
}

impl Session
{
    /// Make a new Session
    pub fn new<S0, S1, S2, S3>(id: S0, secret: S1, user: S2, pass: S3) -> Session
        where
            S0: Into<String>,
            S1: Into<String>,
            S2: Into<String>,
            S3: Into<String>,
    {
        Session
        {
            id: id.into(),
            secret: secret.into(),
            user: user.into(),
            pass: pass.into(),
            token: None,
            expires: Instant::now(),
            remain: 0,
            reset: Instant::now(),
        }
    }

    /// Get a user-agent header for requests
    ///
    /// Reddit will deny any request without a user-agent. It's good practice
    /// to consolidate all the user-agents to the same one in the same place
    pub fn user_agent(&self) -> HeaderValue
    {
        HeaderValue::from_str(&format!("{}/{}", self.user, ::VERSION)).unwrap()
    }

    /// Get a bearer token for reddit
    ///
    /// This function will re-aquire the token if it has expired, or will expire
    /// in 90 seconds.
    pub fn bearer(&mut self, client: &Client) -> Result<String, RedditError>
    {
        if self.token_expired()
        {
            info!("Getting an authorization token");

            let res = client.post("https://www.reddit.com/api/v1/access_token")
                .header(USER_AGENT, self.user_agent())
                .basic_auth(self.id.clone(), Some(self.secret.clone()))
                .body(format!(
                    "grant_type=password\
                    &username={user}\
                    &password={pass}",
                    user = self.user,
                    pass = self.pass))
                .send()?;

            self.update(res.headers()); // does a login response have ratelimit headers?

            match res.status()
            {
                StatusCode::OK => match res.json::<LoginResponse>()
                {
                    Ok(json) =>
                    {
                        self.token = Some(json.access_token);
                        self.expires = Instant::now() + Duration::from_secs(json.expires_in);
                    },
                    Err(_) => return Err(RedditError::BadCredentials)
                },
                StatusCode::UNAUTHORIZED => return Err(RedditError::Unauthorized),
                code => return Err(RedditError::OtherStatus(code))
            }
        }

        Ok(self.token.clone().unwrap())
    }

    /// Prepare for a reddit request
    ///
    /// This function will prepare for a reddit request, and one more
    /// possible request to re-aquire a bearer token.
    pub fn prepare(&self)
    {
        // allow for the request and a possible re-authorization
        if self.remain < 2
        {
            self.wait_for_reset();
        }
    }

    /// Update ratelimit values
    ///
    /// Returns true if the ratelimit values were updated successfully.
    pub fn update(&mut self, headers: &HeaderMap) -> bool
    {
        let rate_remain_header = HeaderName::from_static(
            X_RATELIMIT_REMAINING);
        let rate_reset_header = HeaderName::from_static(
            X_RATELIMIT_RESET);
        let remain = if let Some(remain) = headers.get(rate_remain_header)
        {
            if let Ok(remain) = remain.to_str()
            {
                if let Ok(remain) = remain.parse::<f64>() // why is this a float?
                {
                    remain as u32
                }
                else
                {
                    return false;
                }
            }
            else
            {
                return false;
            }
        }
        else
        {
            return false;
        };

        let reset = if let Some(reset) = headers.get(rate_reset_header)
        {
            if let Ok(reset) = reset.to_str()
            {
                if let Ok(reset) = reset.parse::<u64>()
                {
                    Instant::now() + Duration::from_secs(reset)
                }
                else
                {
                    return false;
                }
            }
            else
            {
                return false;
            }
        }
        else
        {
            return false;
        };

        self.remain = remain;
        self.reset = reset;

        true
    }

    fn token_expired(&self) -> bool
    {
        let now = Instant::now();
        // true if there is no token, the token has expired, or
        // the token will expire in 90 seconds
        self.token.is_none() ||
            now > self.expires ||
            self.expires - now < Duration::from_secs(90)
    }

    fn wait_for_reset(&self)
    {
        let now = Instant::now();
        if now < self.reset
        {
            let sleep = self.reset - now;
            info!("Ratelimit sleeping for {} seconds", sleep.as_secs());

            ::std::thread::sleep(sleep);
        }
    }
}

const X_RATELIMIT_REMAINING: &'static str = "x-ratelimit-remaining";
const X_RATELIMIT_RESET: &'static str     = "x-ratelimit-reset";

#[derive(Deserialize)]
struct LoginResponse
{
    access_token: String,
    expires_in: u64,
}
