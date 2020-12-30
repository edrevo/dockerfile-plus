mod git;
mod http;
mod image;
mod local;

pub use self::git::GitSource;
pub use self::http::HttpSource;
pub use self::image::{ImageSource, ResolveMode};
pub use self::local::LocalSource;

/// Provide an input for other operations. For example: `FROM` directive in Dockerfile.
#[derive(Debug)]
pub struct Source;

impl Source {
    pub fn image<S>(name: S) -> ImageSource
    where
        S: Into<String>,
    {
        ImageSource::new(name)
    }

    pub fn git<S>(url: S) -> GitSource
    where
        S: Into<String>,
    {
        GitSource::new(url)
    }

    pub fn local<S>(name: S) -> LocalSource
    where
        S: Into<String>,
    {
        LocalSource::new(name)
    }

    pub fn http<S>(name: S) -> HttpSource
    where
        S: Into<String>,
    {
        HttpSource::new(name)
    }
}
