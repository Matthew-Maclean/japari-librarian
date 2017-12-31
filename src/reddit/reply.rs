use reqwest::Client;

use super::RedditError;
use super::Session;

pub fn reply(client: &Client, session: &mut Session, replies: Vec<(String, String)>)
    -> Result<(), RedditError>
{
    for (name, body) in replies
    {
        session.prepare();

        let res = client.post("https://oauth.reddit.com/api/comment")
            .header(session.user_agent())
            .header(session.bearer(client)?)
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
