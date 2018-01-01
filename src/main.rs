             extern crate uuid;
             extern crate reqwest;
#[macro_use] extern crate hyper;
             extern crate serde;
#[macro_use] extern crate serde_derive;
             extern crate serde_json;
#[macro_use] extern crate log;
             extern crate simplelog;

use simplelog::*;

mod friend;
mod page;
mod reddit;
mod secrets;
mod process;

fn main()
{
    use std::time::*;

    SimpleLogger::init(LogLevelFilter::Info, Config::default()).unwrap();

    let interval = Duration::from_secs(60); // once a minute

    let client = reqwest::Client::new();
    let mut session = reddit::Session::new(
        secrets::id(),
        secrets::secret(),
        secrets::user(),
        secrets::pass());

    loop
    {
        let start = Instant::now();
        cycle(&client, &mut session);

        let now = Instant::now();
        if start + interval > now
        {
            let sleep = (start + interval) - now;
            info!("sleeping for {} seconds", sleep.as_secs());
            ::std::thread::sleep(sleep);
        }
    }


}

// Todo: replace unrwaps with something more graceful
fn cycle(client: &reqwest::Client, session: &mut reddit::Session)
{
    use reddit::*;
    use process::*;
    use page::{Page, WikiError, partial_page, image_url};

    let messages = match Message::get_unread(client, session, None)
    {
        Ok(m) =>
        {
            info!("Recieved {} unread messages", m.len());
            m
        },
        Err(e) =>
        {
            match e
            {
                RedditError::Unauthorized => error!(
                    "Bad reddit authorization while getting messages"),
                RedditError::BadCredentials => error!(
                    "Bad reddit credentials while getting messages"),
                RedditError::OtherStatus(code) => warn!( // usually a 503 or something
                    "Other status code {:?} while getting messages", code),
                RedditError::OtherError(err) => error!(
                    "Other error \"{:?}\" while getting messages", err),
            }
            return;
        }
    };

    match Message::mark_read(client, session, &messages)
    {
        Ok(_) => info!("Marked messages as read"),
        Err(e) =>
        {
            match e
            {
                RedditError::Unauthorized => error!(
                    "Bad reddit authorization while marking messages"),
                RedditError::BadCredentials => error!(
                    "Bad reddit credentials while marking messages"),
                RedditError::OtherStatus(code) => warn!( // usually a 503 or something
                    "Other status code {:?} while marking messages", code),
                RedditError::OtherError(err) => error!(
                    "Other error \"{:?}\" while marking messages", err),
            }
            return;
        }
    }

    let (pairs, friends) = find_friends(messages, secrets::user());

    info!("Parsed a total {} friends", friends.len());

    let partials = match partial_page::PartialPage::get(client, &friends)
    {
        Ok(p) => p,
        Err(e) =>
        {
            match e
            {
                WikiError::StatusError(code) => warn!(
                    "Other status code {:?} while getting partial pages", code),
                WikiError::RequestError(err) => error!(
                    "Other error \"{:?}\" while getting partial pages", err),
            }
            return;
        }
    };

    let images = match image_url::ImageUrl::get(client, &partials)
    {
        Ok(i) => i,
        Err(e) =>
        {
            match e
            {
                WikiError::StatusError(code) => warn!(
                    "Other status code {:?} while getting images", code),
                WikiError::RequestError(err) => error!(
                    "Other error \"{:?}\" while getting images", err)
            }
            return;
        }
    };

    let pages = Page::make(partials, &images, &friends);

    info!("Made {} pages", pages.len());

    let replies = make_replies(pairs, pages);

    match reply(client, session, replies)
    {
        Ok(_) => info!("Replied to messages"),
        Err(e) => match e
        {
            RedditError::Unauthorized => error!(
                "Bad reddit authorization while replying to messages"),
            RedditError::BadCredentials => error!(
                "Bad reddit credentials while replying to messages"),
            RedditError::OtherStatus(code) => warn!( // usually a 503 or something
                "Other status code {:?} while replying to messages", code),
            RedditError::OtherError(err) => error!(
                "Other error \"{:?}\" while replying to messages", err),
        }
    };
}
