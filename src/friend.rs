use uuid::Uuid;

/// A friend from a media
///
/// Representation of a friend is split into two parts,
/// linked by a UUID. This part is the friend name parsed from messages
/// to the bot.
#[derive(Debug)]
pub struct Friend
{
    /// The name of the friend (formatted)
    ///
    /// name formatting will capitalize the first letter of every word.
    pub name: String,
    /// The media the friend appears in
    ///
    /// Some friends don't appear in some medias, if so, there will be
    /// no wiki page associated with this friend.
    pub media: Media,
    /// The wiki title for this friend
    ///
    /// The wiki formats titles titles as <name>/<media>. This field
    /// may be updated when a request is made to the wiki.
    pub title: String,
    /// The friend ID
    ///
    /// The ID is a lightweight way to have messages and replies keep
    /// track of which friends they contain while still allowing
    /// fields of that friend to be updated.
    pub id: Uuid,
}

impl Friend
{
    pub fn new<S: AsRef<str>>(source: &S, id: Uuid) -> Friend
    {
        // This could probably be done with a simple regex, or nom.
        // But I don't know how.

        let source = source.as_ref();
        // don't format if the first character is a backslash
        if let Some('\\') = source.chars().nth(0)
        {
            let name = source[1..].to_owned();
            return Friend
            {
                name: name.clone(),
                media: Media::None,
                title: name,
                id: id,
            }
        }
        // This is called premature optimization
        let mut name = String::with_capacity(source.len());
        let mut media = String::new();

        let mut chars = source.chars();
        while let Some(c) = chars.next()
        {
            if c == '/'
            {
                break;
            }
            else
            {
                name.push(c);
            }
        }

        for c in chars
        {
            media.push(c);
        }

        let name = Friend::fmt_name(name.trim());
        let media = Media::new(&media);
        let title = format!("{}{}", name, media.wiki_suffix());

        Friend
        {
            name: name,
            media: media,
            title: title,
            id: id,
        }
    }

    fn fmt_name(name: &str) -> String
    {
        let mut fmt = String::with_capacity(name.len());

        let mut first_letter = true;
        for c in name.chars()
        {
            if c.is_alphabetic()
            {
                if first_letter
                {
                    for c in c.to_uppercase()
                    {
                        fmt.push(c);
                    }
                    first_letter = false;
                }
                else
                {
                    for c in c.to_lowercase()
                    {
                        fmt.push(c);
                    }
                }
            }
            else
            {
                first_letter = true;
                fmt.push(c);
            }
        }

        fmt
    }

    /// Searches for a username mention in a source, and parses out
    /// quoted friends
    pub fn find(source: &str, target_user: &str) -> Option<Vec<Friend>>
    {
        // allowed characters in reddit usernames
        static USERNAME_CHARS: &[char] = &[
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
            'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
            'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_', '-',
        ];
        assert!(target_user.chars().all(|c| USERNAME_CHARS.contains(&c)),
            "Target reddit username contained characters that are not allowed in reddit usernames");

        let mut chars = source.chars();
        while let Some(c) = chars.next()
        {
            if c == '/'
            {
                if let Some('u') = chars.next() {
                if let Some('/') = chars.next()
                {
                    let mut user = String::new();
                    while let Some(c) = chars.next()
                    {
                        if USERNAME_CHARS.contains(&c)
                        {
                            user.push(c);
                        }
                        else
                        {
                            // this loop breaks one character after the last username character.
                            // This means that one character is lost. This is OK for reddit
                            // username mentions, because they cannot come directly after
                            // eachother.
                            break;
                        }
                    }

                    if user == target_user
                    {
                        let mut friends = Vec::new();

                        while let Some(c) = chars.next()
                        {
                            if c == '"'
                            {
                                let mut quoted = String::new();
                                while let Some(c) = chars.next()
                                {
                                    if c == '"'
                                    {
                                        break;
                                    }
                                    else
                                    {
                                        quoted.push(c);
                                    }
                                }

                                friends.push(Friend::new(&quoted, Uuid::new_v4()));
                            }
                            else if !c.is_whitespace()
                            {
                                break;
                            }
                        }

                        return Some(friends)
                    }
                }}
            }
        }
        
        None
    }
}

/// A media a friend might appear in.
///
/// Parsing of a media is forgiving, and falls back to none.
#[derive(Debug, Copy, Clone)]
pub enum Media
{
    /// No media specified, or the media could not be parsed
    None,
    /// The 2017 anime (and any future series in that continuity)
    Anime,
    /// All manga adaptations
    Manga,
    /// The nexon game
    Nexon,
    /// All stage adaptations
    Stage,
    /// The upcoming pavilion game
    Pavilion,
}

impl Media
{
    /// Parse a new Media from a string
    pub fn new<S: AsRef<str>>(source: &S) -> Media
    {
        match source.as_ref().trim().to_lowercase().as_str()
        {
            "anime" => Media::Anime,
            "manga" => Media::Manga,
            "nexon" | "nexon game" => Media::Nexon,
            "stage" | "stage play" => Media::Stage,
            "pavilion" => Media::Pavilion,
            _ => Media::None,
        }
    }
    
    /// Format the media into it's wiki suffix (includes the slash)
    pub fn wiki_suffix(&self) -> &'static str
    {
        match self
        {
            &Media::None => "",
            &Media::Anime => "/Anime",
            &Media::Manga => "/Manga",
            &Media::Nexon => "/Nexon Game",
            &Media::Stage => "/Stage Play",
            &Media::Pavilion => "/Pavilion",
        }
    }
}
