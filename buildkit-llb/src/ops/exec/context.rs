use std::iter::once;
use std::path::PathBuf;

use buildkit_proto::pb::Meta;

#[derive(Debug, Clone)]
pub(crate) struct Context {
    pub name: String,
    pub args: Vec<String>,
    pub env: Vec<String>,

    pub cwd: PathBuf,
    pub user: String,
}

impl Context {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),

            cwd: PathBuf::from("/"),
            user: "root".into(),

            args: vec![],
            env: vec![],
        }
    }
}

impl Into<Meta> for Context {
    fn into(self) -> Meta {
        Meta {
            args: {
                once(self.name.clone())
                    .chain(self.args.iter().cloned())
                    .collect()
            },

            env: self.env,
            cwd: self.cwd.to_string_lossy().into(),
            user: self.user,

            ..Default::default()
        }
    }
}
