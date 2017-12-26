macro_rules! secret_file
{
    ($name:ident, $path:expr, $trail:expr) =>
    {
        #[inline(always)]
        pub fn $name() -> &'static str
        {
            let contents = include_str!($path);
            &contents[0..contents.len() - $trail]
        }
    }
}

// these files are assumed to have one trailing newline
secret_file!(id,     "../.secrets/id.txt",     1); // client ID
secret_file!(secret, "../.secrets/secret.txt", 1); // client secret
secret_file!(user,   "../.secrets/user.txt",   1); // account username
secret_file!(pass,   "../.secrets/pass.txt",   1); // account password
