pub mod partial_page;
pub mod image_url;

use uuid::Uuid;

use friend::Friend;

/// A wiki page
///
/// A Page is usually for a friend, but actually describes many pages on the wiki.  
/// The page is built in two requests, one to get the page title and URL. And
/// another to get the image URL.
#[derive(Debug, Clone)]
pub struct Page
{
    /// The friends that this page is linked to
    ///
    /// Multiple friends might parse differently, but link
    /// to the same page. So we keep track of all of them.
    pub friends: Vec<Uuid>,
    /// The title of the page
    pub title: String,
    /// The URL of the page
    pub url: String,
    /// The URL of the image, if any
    pub image: Option<String>,
}

impl Page
{
    pub fn make(
        partials: Vec<partial_page::PartialPage>,
        images: &[image_url::ImageUrl],
        friends: &[Friend])
        -> Vec<Page>
    {
        partials.into_iter()
            .map(|partial|
            {
                let image_url = Page::find_image(&partial.image_title, images);
                let ids = Page::find_friends(&partial.title, &partial.aliases, friends);

                Page
                {
                    friends: ids,
                    title: partial.title,
                    url: partial.url,
                    image: image_url,
                }
            })
            .collect::<Vec<_>>()
    }

    fn find_image(title: &Option<String>, images: &[image_url::ImageUrl]) -> Option<String>
    {
        let title = match title
        {
            &Some(ref title) => title,
            &None => return None,
        };

        for image in images
        {
            if &image.title == title
            {
                return Some(image.url.clone())
            }
        }

        None
    }

    fn find_friends(title: &str, aliases: &[String], friends: &[Friend]) -> Vec<Uuid>
    {
        let mut ids = Vec::new();

        for friend in friends
        {
            if friend.title == title
            {
                ids.push(friend.id);
            }
            else if aliases.contains(&friend.title)
            {
                ids.push(friend.id);
            }
        }

        ids
    }
}

/// The user-agent to use for the wiki
pub static USER_AGENT: &'static str = "japari-librarian/0.0.1";
/// The maximum number of titles that can be put into one wiki request
///
/// The actual number is 50, but due to wiki weirdness and the high
/// likleyhood of off-by-one errors in my code, I've set it at 45.
pub static MAX_TITLES: usize = 45;

use reqwest::{StatusCode, Error};

/// An error that might occur when making a wiki request
#[derive(Debug)]
pub enum WikiError
{
    StatusError(StatusCode),
    RequestError(Error),
}

impl From<Error> for WikiError
{
    fn from(err: Error) -> WikiError
    {
        WikiError::RequestError(err)
    }
}
