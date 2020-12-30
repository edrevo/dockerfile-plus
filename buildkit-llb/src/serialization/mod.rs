use std::collections::BTreeMap;

mod id;
mod operation;
mod output;

pub(crate) use self::id::OperationId;
pub(crate) use self::operation::Operation;
pub(crate) use self::output::Node;

pub(crate) type Result<T> = std::result::Result<T, ()>;

#[derive(Default)]
pub struct Context {
    inner: BTreeMap<u64, Node>,
}

impl Context {
    #[allow(clippy::map_entry)]
    pub(crate) fn register<'a>(&'a mut self, op: &dyn Operation) -> Result<&'a Node> {
        let id = **op.id();

        if !self.inner.contains_key(&id) {
            let node = op.serialize(self)?;
            self.inner.insert(id, node);
        }

        Ok(self.inner.get(&id).unwrap())
    }

    #[cfg(test)]
    pub(crate) fn registered_nodes_iter(&self) -> impl Iterator<Item = &Node> {
        self.inner.iter().map(|pair| pair.1)
    }

    pub(crate) fn into_registered_nodes(self) -> impl Iterator<Item = Node> {
        self.inner.into_iter().map(|pair| pair.1)
    }
}
