use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;

use super::RedditError;
use super::Session;

pub fn reply(client: &Client, session: &mut Session, replies: Vec<(String, String)>)
    -> Result<(), RedditError>
{
    for (name, body) in replies
    {
        session.prepare();

        let res = client.post("https://oauth.reddit.com/api/comment")
            .header(USER_AGENT, session.user_agent())
            .bearer_auth(session.bearer(client)?)
            .body(format!(
                "parent={name}\
                &text={body}",
                name = name,
                body = body))
            .send()?;

        session.update(res.headers());
    }

    Ok(())
}
