use std::fmt::Debug;

use super::{Context, OperationId};
use super::{Node, Result};

pub(crate) trait Operation: Debug + Send + Sync {
    fn id(&self) -> &OperationId;

    fn serialize(&self, cx: &mut Context) -> Result<Node>;
}
