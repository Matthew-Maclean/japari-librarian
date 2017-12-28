use reqwest::Client;

use super::{RedditError, Session};

/// A reddit inbox message
#[derive(Debug, Deserialize)]
pub struct Message
{
    /// The fullname of the message
    pub name: String,
    /// The author of the message
    ///
    /// Kept for loggin purposes
    pub author: String,
    /// If the message was a comment or comment reply, the subreddit it was posted in,
    /// other wise `None`
    pub subreddit: Option<String>,
    /// The body of the message
    pub body: String,
}

impl Message
{
    /// Get a number of unread messages (default is all)
    pub fn get_unread(client: &Client, session: &mut Session, limit: Option<u64>)
        -> Result<Vec<Message>, RedditError>
    {
        static MAX_MESSAGES: u64 = 100; // the maximum messages per request from reddit

        fn get_messages(
            client: &Client,
            session: &mut Session,
            after: Option<String>,
            limit: u64) // it is expected that the caller will keep track of the limit
            -> Result<Vec<Message>, RedditError>
        {
            session.prepare();

            let url = if let Some(after) = after
            {
                ::reqwest::Url::parse_with_params(
                    "https://oauth.reddit.com/message/unread/.json",
                    &[
                        ("after", &after),
                        ("limit", &limit.to_string())
                    ]).unwrap()
            }
            else
            {
                ::reqwest::Url::parse_with_params(
                    "https://oauth.reddit.com/message/unread/.json",
                    &[("limit", &limit.to_string())]).unwrap()
            };

            let mut res = client.get(url)
                .header(session.user_agent())
                .header(session.bearer(client)?)
                .send()?;

            use reqwest::StatusCode;

            match res.status()
            {
                StatusCode::Ok => Ok(res.json::<MessageResponse>()?.data.children.into_iter()
                    .map(|x| x.data)
                    .collect::<Vec<_>>()),
                StatusCode::Unauthorized => Err(RedditError::Unauthorized),
                code => Err(RedditError::OtherStatus(code))
            }
        }

        let mut messages: Vec<Message> = Vec::new();

        match limit
        {
            Some(lim) => while (messages.len() as u64) < lim
            {
                let after = if messages.len() != 0
                {
                    Some(messages[messages.len() - 1].name.clone())
                }
                else
                {
                    None
                };

                let mut msgs = if lim - (messages.len() as u64) < MAX_MESSAGES
                {
                    get_messages(client, session, after, lim - messages.len() as u64)?
                }
                else
                {
                    get_messages(client, session, after, MAX_MESSAGES)?
                };

                if msgs.len() == 0
                {
                    break;
                }

                messages.append(&mut msgs);
            },
            None => loop
            {
                let after = if messages.len() != 0
                {
                    Some(messages[messages.len() - 1].name.clone())
                }
                else
                {
                    None
                };

                let mut msgs = get_messages(client, session, after, MAX_MESSAGES)?;

                if msgs.len() == 0
                {
                    break;
                }

                messages.append(&mut msgs);
            },
        }

        Ok(messages)
    }

    /// Mark a series of messages as read
    pub fn mark_read(client: &Client, session: &mut Session, messages: &[Message])
        -> Result<(), RedditError>
    {
        if messages.len() == 0
        {
            return Ok(());
        }

        let mut names = String::new();

        for msg in messages
        {
            names.push_str(&msg.name);
            names.push(',');
        }

        session.prepare();

        let res = client.post("https://oauth.reddit.com/api/read_message")
            .header(session.user_agent())
            .header(session.bearer(client)?)
            .body(format!("id={}", names))
            .send()?;

        use reqwest::StatusCode;

        match res.status()
        {
            StatusCode::Ok => Ok(()),
            StatusCode::Unauthorized => Err(RedditError::Unauthorized),
            code => Err(RedditError::OtherStatus(code))
        }
    }
}

#[derive(Deserialize)]
struct MessageResponse{ data: MessageList }
#[derive(Deserialize)]
struct MessageList{ children: Vec<MessageContainer> }
#[derive(Deserialize)]
struct MessageContainer{ data: Message }
