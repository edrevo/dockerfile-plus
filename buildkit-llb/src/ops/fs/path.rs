use std::path::{Path, PathBuf};

use crate::utils::{OperationOutput, OwnOutputIdx};

/// Internal representation for not yet specified path.
#[derive(Debug)]
pub struct UnsetPath;

/// Operand of *file system operations* that defines either source or destination layer and a path.
#[derive(Debug)]
pub enum LayerPath<'a, P: AsRef<Path>> {
    /// References one of the *current operation outputs* and a path.
    Own(OwnOutputIdx, P),

    /// References an *output of another operation* and a path.
    Other(OperationOutput<'a>, P),

    /// A path in an *empty* layer (equivalent of Dockerfile's scratch source).
    Scratch(P),
}

impl<'a, P: AsRef<Path>> LayerPath<'a, P> {
    /// Transform the layer path into owned variant (basically, with `PathBuf` as the path).
    pub fn into_owned(self) -> LayerPath<'a, PathBuf> {
        use LayerPath::*;

        match self {
            Other(input, path) => Other(input, path.as_ref().into()),
            Own(output, path) => Own(output, path.as_ref().into()),
            Scratch(path) => Scratch(path.as_ref().into()),
        }
    }
}
