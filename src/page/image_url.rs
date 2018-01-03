use reqwest::Client;

use super::WikiError;
use super::partial_page::PartialPage;

/// An image URL page
#[derive(Debug)]
pub struct ImageUrl
{
    pub title: String,
    pub url: String,
}

impl ImageUrl
{
    pub fn get(client: &Client, partials: &[PartialPage]) -> Result<Vec<ImageUrl>, WikiError>
    {
        let mut images = Vec::with_capacity(partials.len());
        let mut index = 0;

        while partials.len() - index > super::MAX_TITLES
        {
            images.append(&mut ImageUrl::make_request(
                client,
                &partials[index..(index + super::MAX_TITLES)])?);
            index += super::MAX_TITLES;
        }

        images.append(&mut ImageUrl::make_request(
                client,
                &partials[index..])?);

        Ok(images)
    }

    fn make_request(client: &Client, partials: &[PartialPage]) -> Result<Vec<ImageUrl>, WikiError>
    {
        assert!(partials.len() <= super::MAX_TITLES);

        use reqwest::{Url, StatusCode};
        use reqwest::header::UserAgent;

        let titles = ImageUrl::make_titles(partials);

        let url = Url::parse_with_params("https://japari-library.com/w/api.php",
            &[
                ("action", "query"),
                ("format", "json"),
                ("prop", "imageinfo"),
                ("iiprop", "url"),
                ("titles", &titles)
            ]).unwrap();

        let mut res = client.get(url)
            .header(UserAgent::new(super::user_agent()))
            .send()?;

        match res.status()
        {
            StatusCode::Ok => ImageUrl::parse_response(res.json()?),
            code => Err(WikiError::StatusError(code)),
        }
    }

    fn make_titles(partials: &[PartialPage]) -> String
    {
        let mut s = String::new();
        s.push('|'); // this makes sure we have at least one title to query
                     // otherwise the wiki doesn't even bother with a 'query'
                     // element, and the we don't handle that.
        for partial in partials
        {
            if let &Some(ref ititle) = &partial.image_title
            {
                s.push_str(ititle);
                s.push('|');
            }
        }
        s
    }

    fn parse_response(mut res: Response) -> Result<Vec<ImageUrl>, WikiError>
    {
        let mut images = Vec::new();

        for (_, image) in res.query.pages.drain()
        {
            if image.missing.is_none() & image.invalid.is_none()
            {
                let title = image.title
                    .expect("Valid and non-missing imagedata did not have a 'title'");
                let imageinfo = image.imageinfo
                    .expect("Valid and non-missing imagedata did not have an 'imageinfo'");

                if imageinfo.len() != 0
                {
                    images.push(ImageUrl
                    {
                        title: title,
                        url: imageinfo[0].url.clone(),
                    });
                }
            }
        }

        Ok(images)
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
    pages: HashMap<String, Image>
}

#[derive(Deserialize)]
struct Image
{
    title: Option<String>,
    imageinfo: Option<Vec<ImageInfo>>,
    missing: Option<String>,
    invalid: Option<String>,
}

#[derive(Deserialize)]
struct ImageInfo
{
    url: String,
}
