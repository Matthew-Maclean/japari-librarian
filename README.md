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

- The 2017 anime "anime" or "season 1"  
- The 2019 anime season 2 "season 2"  
- Either manga series "manga" 
- Kemono Friends Festival "festival"
- Kemono Friends Pavilion "pavilion"
- Kemono Friends 3 (either phone or arcade) "kf3"
- The original Nexon game "nexon game" or "nexon"  
- Any of the stage performances "stage play" or "stage"  

It will respond (if it can parse the friend, and find the page) with a library link, and an
image if it can find one.

## How to use it

Japari Librarian runs on a loop, checking reddit and processing/responding to messages every
interval. It logs to stdout, and optionally logs redirects "info" and below level log messages
to a log file. Start it like this:

    japari-librarian -i<Interval> [-f<LogFile>]

For example, to loop once a minute, and log to a file called `log.txt`:

    japari-librarian -i 60 -f log.txt

Or to loop once every 30 seconds, and log everything to stdout:

    japari-librarian -i 30

---

## FAQ

### No one has asked you questions, how can any be "frequently asked"?

### Why doesn't it respond with an excerpt from the page?

The Japari Library doesn't have the "Extracts" extension, and the actual wiki-text is not
formatted consistently, so I can't reliably just grab a section from the page.

### How come it sometimes pulls an image that isn't the main one, or isn't an image at all?

There's no way to tell specifically which image on a page is the "main" one, so this program
looks for images with the name of the page in their name, or the string "original". This
doesn't always work, but it's pretty good in my testing.

---

## How to repurpose for your own uses

1. Change refrences to the Japrai Library in the `page` module to a wiki
    of your choice. The code should work with any wikimedia-powered wiki.

2. Modify the `Friend` type to your liking. Although it is called "Friend",
    after the characters in Kemono Friends, it really just represents a way to parse
    and store information from a message.

3. Fix your `secrets` module. The `secrets` module has four files that it loads
    from, but as long as the four functions are present with the right signature,
    everything should work. If you keep the file loading method, you need these four
    files in the same directory as `src` (but not in `src`):

    - `.secrets/id.txt`: Contains the reddit bot client ID.
    - `.secrets/secret.txt`: Contains the reddit bot client secret.
    - `.secrets/user.txt`: The reddit account username that the bot will use.
    - `.secrets/pass.txt`: The reddit account password that the bot will use.

    All of these files are assumed to have a trailing newline. The `$trail` argument
    decides how many character from the end to strip. If you don't want to use a
    `.secrets` folder, just make sure these functions are present:

    - `fn id() -> &'static str`
    - `fn secret() -> &'static str`
    - `fn user() -> &'static str`
    - `fn pass() -> &'static str`

    With their usual meanings

4. Change `MAINTAINER` in `main.rs` to your username, so people bother you rather than me.

5. Change the subreddit whitelist in `filter_messages`, or change the whole function to suit
    your needs.
