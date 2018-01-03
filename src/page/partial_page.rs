use reqwest::Client;

use friend::Friend;
use super::WikiError;

/// A partially made page
#[derive(Debug)]
pub struct PartialPage
{
    /// The title of the page
    pub title: String,
    /// Any titles that were normalized to this page's title
    pub aliases: Vec<String>,
    /// The URL of the page
    pub url: String,
    /// The image URL, if any
    pub image_title: Option<String>,
}

impl PartialPage
{
    pub fn get(client: &Client, friends: &[Friend]) -> Result<Vec<PartialPage>, WikiError>
    {
        let mut partials = Vec::with_capacity(friends.len());
        let mut index = 0;

        while friends.len() - index > super::MAX_TITLES
        {
            partials.append(&mut PartialPage::make_request(
                client,
                &friends[index..(index + super::MAX_TITLES)])?);
            index += super::MAX_TITLES;
        }

        partials.append(&mut PartialPage::make_request(
                client,
                &friends[index..])?);

        Ok(partials)
    }

    fn make_request(client: &Client, friends: &[Friend]) -> Result<Vec<PartialPage>, WikiError>
    {
        assert!(friends.len() <= super::MAX_TITLES);

        use reqwest::{Url, StatusCode};
        use reqwest::header::UserAgent;

        let titles = PartialPage::make_titles(&friends);

        let url = Url::parse_with_params("https://www.japari-library.com/w/api.php",
            &[
                ("action", "query"),
                ("format", "json"),
                ("prop", "images|info"),
                ("inprop", "url"),
                ("imlimit", "500"),
                ("titles", &titles),
            ]).unwrap();

        let mut res = client.get(url)
            .header(UserAgent::new(super::USER_AGENT))
            .send()?;

        match res.status()
        {
            StatusCode::Ok => Ok(PartialPage::parse_response(res.json()?)),
            code => Err(WikiError::StatusError(code))
        }
    }

    fn make_titles(friends: &[Friend]) -> String
    {
        let mut s = String::new();
        // for some reason, if we ask the wiki for a weird page like "Main Page"
        // it sometimes gives us a list rather than a map.
        // I don't know why this is, but we can prevent it by always requesting
        // at least one regular page, like "Serval"
        s.push_str("Serval|");
        for friend in friends
        {
            s.push_str(&friend.title);
            s.push('|');
        }
        s
    }

    fn parse_response(res: Response) -> Vec<PartialPage>
    {
        let mut query = res.query;

        let normalized = query.normalized;

        let mut partials = Vec::new();
        for (_, page) in query.pages.drain()
        {
            match PartialPage::parse_page(page, &normalized)
            {
                Some(partial) => partials.push(partial),
                None => {}, // maybe log it?
            }
        }
        partials
    }

    fn parse_page(page: PageJSON, normalized: &Option<Vec<Normalized>>) -> Option<PartialPage>
    {
        if page.invalid.is_some() || page.missing.is_some()
        {
            None
        }
        else
        {
            let title = page.title?;
            let url = page.fullurl?;

            let aliases = PartialPage::get_aliases(&title, normalized);
            let image_title = PartialPage::select_image(&title, &page.images);

            Some(PartialPage
            {
                title: title,
                aliases: aliases,
                url: url,
                image_title: image_title,
            })
        }
    }

    fn get_aliases(title: &str, normalized: &Option<Vec<Normalized>>) -> Vec<String>
    {
        if let &Some(ref normalized) = normalized
        {
            normalized.iter()
                .filter(|n| &n.to == title)
                .map(|n| n.from.clone())
                .collect::<Vec<_>>()
        }
        else
        {
            Vec::new()
        }
    }

    fn select_image(title: &str, images: &Option<Vec<Image>>) -> Option<String>
    {
        // common image extentions
        let exts = [
            "jpg", "png", "jpeg",
            "gif", "bmp", "tiff"
        ];

        if let &Some(ref images) = images
        {
            let mut selected = None;

            for image in images.iter()
                .map(|image| image.title.clone()) // just the titles
                .filter(|image| exts.iter() // only images
                        .any(|ext| image.to_lowercase().ends_with(ext)))
            {
                // initially set the selected image to the first one
                if selected.is_none()
                {
                    selected = Some(image.clone());
                }

                let lc = image.to_lowercase();
                // if the title contains "original", select it and
                // don't continue
                if lc.contains("original")
                {
                    selected = Some(image);
                    break;
                }
                // if the title contains the page title, select it,
                // but continue in case another one later on is a 
                // better matc
                if lc.contains(&title.to_lowercase())
                    && selected.is_none()
                {
                    selected = Some(image);
                }
            }

            return selected;
        }

        None
    }
}

// ==============================
// Serde structs below
// ==============================

#[derive(Deserialize)]
struct Response
{
    query: Query,
}

use std::collections::HashMap;

#[derive(Deserialize)]
struct Query
{
    normalized: Option<Vec<Normalized>>,
    pages: HashMap<String, PageJSON>,
}

#[derive(Deserialize)]
struct Normalized
{
    from: String,
    to: String,
}

#[derive(Deserialize)]
struct PageJSON
{
    title: Option<String>,
    images: Option<Vec<Image>>,
    fullurl: Option<String>,
    missing: Option<String>,
    invalid: Option<String>,
}

#[derive(Deserialize)]
struct Image
{
    title: String,
}
