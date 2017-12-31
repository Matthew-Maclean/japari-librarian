use uuid::Uuid;

use reddit::Message;
use friend::Friend;
use page::Page;

/// Find friends in messages
pub fn find_friends(messages: Vec<Message>, user: &str) -> (Vec<(Message, Vec<Uuid>)>, Vec<Friend>)
{
    let mut message_ids = Vec::new();
    let mut friends = Vec::new();

    for message in messages
    {
        if let Some(mut found) = Friend::find(&message.body, user)
        {
            message_ids.push((message, found.iter().map(|f| f.id).collect::<Vec<_>>()));

            friends.append(&mut found);
        }
    }

    (message_ids, friends)
}

/// Format replies to messages
pub fn make_replies(messages: Vec<(Message, Vec<Uuid>)>, pages: Vec<Page>) -> Vec<(String, String)>
{
    let mut replies = Vec::new();

    for (message, ids) in messages
    {
        let mut fmt = String::new();
        for page in pages.iter()
        {
            if page.friends.iter().any(|id| ids.contains(id))
            {
                fmt.push_str(&format!("[{title}]({link})",
                    title = escape_md(&page.title),
                    link = page.url));

                if let &Some(ref image) = &page.image
                {
                    fmt.push_str(&format!(" ([pic]({}))", image));
                }

                fmt.push_str("\n\n"); // two newlines for one visible newline
            }
        }

        fmt.push_str(
            "---\n\n^^I'm ^^a ^^bot ^^friend! ^^Message ^^\\/u/YourGamerMom \
            ^^if ^^you ^^have ^^questions ^^or ^^concerns. ^^Check ^^out ^^my \
            ^^[code](https://www.example.com), ^^and ^^my \
            ^^[subreddit](https://www.reddit.com/r/japari_librarian)");

        replies.push((message.name, fmt));
    }

    replies
}

// kind of rudimentary, but probably OK
// I'm not super worried about XSS with markdown
fn escape_md(source: &str) -> String
{
    let special = ['\\', '`', '*', '_', '#']; // maybe more
    let escape = '\\';

    let mut fmt = String::with_capacity(source.len());

    for c in source.chars()
    {
        if special.contains(&c)
        {
            fmt.push(escape);
        }
        fmt.push(c);
    }

    fmt
}
