             extern crate uuid;
             extern crate reqwest;
#[macro_use] extern crate hyper;
             extern crate serde;
#[macro_use] extern crate serde_derive;
             extern crate serde_json;
#[macro_use] extern crate log;
             extern crate simplelog;
             extern crate clap;

use simplelog::*;

mod friend;
mod page;
mod reddit;
mod secrets;
mod process;

/// The current version
pub static VERSION: &'static str = "1.0";
/// The maintainer username
pub static MAINTAINER: &'static str = "YourGamerMom";

fn main()
{
    use std::time::*;
    use std::fs::OpenOptions;

    let matches = clap::App::new("japari-libraria")
        .version(VERSION)
        .author("Matthew Maclean")
        .about("Reddit bot for /r/KemonoFriends")
        .arg(clap::Arg::with_name("logfile")
             .short("f")
             .long("logfile")
             .takes_value(true)
             .value_name("LOGFILE")
             .required(false)
             .help("If set, the log file where info and above level logs go"))
        .arg(clap::Arg::with_name("interval")
             .short("i")
             .long("interval")
             .takes_value(true)
             .value_name("INTERVAL")
             .required(true)
             .help("The interval, in seconds, to run the program loop on"))
        .get_matches();

    if let Some(logfile) = matches.value_of("logfile")
    {
         CombinedLogger::init(vec![
            SimpleLogger::new(LogLevelFilter::Warn, Config::default()),
            WriteLogger::new(LogLevelFilter::Info, Config::default(), OpenOptions::new()
                .append(true)
                .create(true)
                .open(logfile).unwrap()),
        ]).unwrap();
    }
    else
    {
        SimpleLogger::init(LogLevelFilter::Info, Config::default()).unwrap();
    }

    let interval = Duration::from_secs(matches.value_of("interval")
        .unwrap()
        .parse::<u64>()
        .unwrap());

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
        Ok(ref m) if m.len() == 0 =>
        {
            info!("No unread messages");
            return;
        }
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

    let messages = filter_messages(messages);

    info!("Filtered to {} messages", messages.len());

    let (pairs, friends) = find_friends(messages, secrets::user());

    if friends.len() == 0
    {
        info!("No friends found");
        return;
    }

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

    info!("Made {} pages", pages.len()); // due to wiki wierdness, it always makes at least one

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

fn filter_messages(messages: Vec<reddit::Message>) -> Vec<reddit::Message>
{
    // subreddit whitelist
    let whitelist = &[
        "kemonofriends",
        "japari_librarian"
    ];

    let mut filtered = Vec::with_capacity(messages.len());
    for msg in messages
    {
        // allow any private message, or any comment from the whitelisted subreddits
        if let &Some(ref sub) = &msg.subreddit
        {
            if !whitelist.contains(&sub.to_lowercase().as_str())
            {
                continue;
            }
        }

        filtered.push(msg);
    }

    filtered
}
