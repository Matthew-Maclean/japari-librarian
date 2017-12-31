             extern crate uuid;
             extern crate reqwest;
#[macro_use] extern crate hyper;
             extern crate serde;
#[macro_use] extern crate serde_derive;
             extern crate serde_json;

mod friend;
mod page;
mod reddit;
mod secrets;
mod process;

fn main()
{
    println!("todo");
}

// Todo: replace unrwaps with something more graceful
fn cycle(client: &reqwest::Client, session: &mut reddit::Session)
{
    use reddit::*;
    use process::*;
    use page::{Page, partial_page, image_url};

    let messages = Message::get_unread(client, session, None).unwrap();

    Message::mark_read(client, session, &messages).unwrap();

    let (pairs, friends) = find_friends(messages, secrets::user());

    let partials = partial_page::PartialPage::get(client, &friends).unwrap();

    let images = image_url::ImageUrl::get(client, &partials).unwrap();

    let pages = Page::make(partials, &images, &friends);

    let replies = make_replies(pairs, pages);

    reply(client, session, replies).unwrap();
}
