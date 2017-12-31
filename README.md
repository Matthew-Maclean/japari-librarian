# japari-librarian
A reddit bot for [/r/KemonoFriends](https://www.reddit.com/r/KemonoFriends).

Find the test subreddit at [/r/japari_librarian](https://www.reddit.com/r/japari_librarian).

---

## What it does

Japari-librarian responds to comments that mention it (or direct messages), and tries to provide
links and images from the [Japari Library](https://en.wikipedia.org/wiki/Rascal_the_Raccoon)
based on friends mentioned in the comments. Call it like this:

> /u/japari-librarian "Kaban" "Serval/Anime"

This will prompt japari-librarian to try to link the the library pages for "Kaban", and "Serval" as
she appeared in the 2017 anime. The specific format is this:

    /u/japari-librarian ["<friend>/<media>" | "<friend>"]

The `friend` will be formatted to capitalize the first letter of every word, and the `media`
will be matched against a known set of media unless the first character in the quotes is a
backslash. The known medias are:

- The 2017 anime "anime"  
- Either manga series "manga"  
- The original Nexon game "nexon game" or "nexon"  
- Any of the stage performances "stage play" or "stage"  
- The upcoming Pavilion game "pavilion"

It will respond (if it can parse the friend, and find the page) with a library link, and an
image if it can find one.

---

## FAQ

### No one has asked you questions, how can any be "frequently asked"

### Why doesn't it respond with an excerpt from the page?

The Japari Library doesn't have the "Extracts" extension, and the actual wiki-text is not
formatted consistently, so I can't reliably just grab a section from the page.

---

## How to repurpose for your own uses

1. Change refrences to the Japrai Library in the `page` module to a wiki
of your choice. The code should work with any wikimedia-powered wiki.

2. Modify the `Friend` type to your liking. Although it is called "Friend",
after the characters in Kemono Friends, it really just represents a way to parse
and store information from a message.

3. Create your `secrets` directory. There should be four files: `id.txt`,
`secret.txt`, `user.txt`, and `pass.txt`. These respectivley contain your
reddit bot's client ID, client secrets, and reddit username and password.
These files are assumed to have a trailing newline, look around the
`secrets.rs` to find out more.

4. In `process.rs`, change the reference to `/u/YourGamerMom` with a reference to
your reddit account.
