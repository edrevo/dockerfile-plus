use std::sync::Arc;

pub mod exec;
pub mod fs;
pub mod source;
pub mod terminal;

pub use self::exec::Command;
pub use self::fs::FileSystem;
pub use self::source::Source;
pub use self::terminal::Terminal;

use crate::utils::OperationOutput;

pub trait MultiBorrowedOutput<'a> {
    fn output(&'a self, number: u32) -> OperationOutput<'a>;
}

pub trait MultiBorrowedLastOutput<'a> {
    fn last_output(&'a self) -> Option<OperationOutput<'a>>;
}

pub trait MultiOwnedOutput<'a> {
    fn output(&self, number: u32) -> OperationOutput<'a>;
}

pub trait MultiOwnedLastOutput<'a> {
    fn last_output(&self) -> Option<OperationOutput<'a>>;
}

pub trait SingleBorrowedOutput<'a> {
    fn output(&'a self) -> OperationOutput<'a>;
}

pub trait SingleOwnedOutput<'a> {
    fn output(&self) -> OperationOutput<'a>;
}

/// Common operation methods.
pub trait OperationBuilder<'a> {
    /// Sets an operation display name.
    fn custom_name<S>(self, name: S) -> Self
    where
        S: Into<String>;

    /// Sets caching behavior.
    fn ignore_cache(self, ignore: bool) -> Self;

    /// Convert the operation into `Arc` so it can be shared when efficient borrowing is not possible.
    fn ref_counted(self) -> Arc<Self>
    where
        Self: Sized + 'a,
    {
        Arc::new(self)
    }
}
