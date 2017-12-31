# japari-librarian
A reddit bot for /r/KemonoFriends

Find the test subreddit at [/r/japari_librarian](https://www.reddit.com/r/japari_librarian).

---

## How to repurpose for your own uses

1. Change refrences to the Japrai Library in the `page` module to a wiki
of your choice. The code should work with any wikimedia-powered wiki.

2. Modify the `Friend` type to you liking. Although it is called "Friend",
after the characters in Kemono Friends, it really just represents a way to parse
and store information from a message.

3. Create your `secrets` directory. There should be four files: `id.txt`,
`secret.txt`, `user.txt`, and `pass.txt`. These respectivley contain your
reddit bot's client ID, client secrets, and reddit username and password.
These files are assumed to have a trailing newline, look around the
`secrets.rs` to find out more.
