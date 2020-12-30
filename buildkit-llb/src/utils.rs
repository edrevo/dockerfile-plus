use std::sync::Arc;

use crate::serialization::Operation;

#[derive(Copy, Clone, Debug)]
pub struct OutputIdx(pub u32);

#[derive(Copy, Clone, Debug)]
pub struct OwnOutputIdx(pub u32);

#[derive(Debug, Clone)]
pub struct OperationOutput<'a> {
    kind: OperationOutputKind<'a>,
}

#[derive(Debug, Clone)]
enum OperationOutputKind<'a> {
    Owned(Arc<dyn Operation + 'a>, OutputIdx),
    Borrowed(&'a dyn Operation, OutputIdx),
}

impl<'a> OperationOutput<'a> {
    pub(crate) fn owned(op: Arc<dyn Operation + 'a>, idx: OutputIdx) -> Self {
        Self {
            kind: OperationOutputKind::Owned(op, idx),
        }
    }

    pub(crate) fn borrowed(op: &'a dyn Operation, idx: OutputIdx) -> Self {
        Self {
            kind: OperationOutputKind::Borrowed(op, idx),
        }
    }

    pub(crate) fn operation(&self) -> &dyn Operation {
        match self.kind {
            OperationOutputKind::Owned(ref op, ..) => op.as_ref(),
            OperationOutputKind::Borrowed(ref op, ..) => *op,
        }
    }

    pub(crate) fn output(&self) -> OutputIdx {
        match self.kind {
            OperationOutputKind::Owned(_, output) | OperationOutputKind::Borrowed(_, output) => {
                output
            }
        }
    }
}

impl Into<i64> for OutputIdx {
    fn into(self) -> i64 {
        self.0.into()
    }
}
impl Into<i64> for &OutputIdx {
    fn into(self) -> i64 {
        self.0.into()
    }
}

impl Into<i64> for OwnOutputIdx {
    fn into(self) -> i64 {
        self.0.into()
    }
}
impl Into<i64> for &OwnOutputIdx {
    fn into(self) -> i64 {
        self.0.into()
    }
}

impl Into<i32> for OutputIdx {
    fn into(self) -> i32 {
        self.0 as i32
    }
}
impl Into<i32> for &OutputIdx {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

impl Into<i32> for OwnOutputIdx {
    fn into(self) -> i32 {
        self.0 as i32
    }
}
impl Into<i32> for &OwnOutputIdx {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

#[cfg(test)]
pub mod test {
    #[macro_export]
    macro_rules! check_op {
        ($op:expr, $(|$name:ident| $value:expr,)*) => ($crate::check_op!($op, $(|$name| $value),*));
        ($op:expr, $(|$name:ident| $value:expr),*) => {{
            #[allow(unused_imports)]
            use crate::serialization::{Context, Operation};

            let mut context = Context::default();
            let serialized = $op.serialize(&mut context).unwrap();

            $(crate::check_op_property!(serialized, context, $name, $value));*
        }};
    }

    #[macro_export]
    macro_rules! check_op_property {
        ($serialized:expr, $context:expr, op, $value:expr) => {{
            use std::io::Cursor;

            use buildkit_proto::pb;
            use prost::Message;

            assert_eq!(
                pb::Op::decode(Cursor::new(&$serialized.bytes)).unwrap().op,
                Some($value)
            );
        }};

        ($serialized:expr, $context:expr, inputs, $value:expr) => {{
            use std::io::Cursor;

            use buildkit_proto::pb;
            use prost::Message;

            assert_eq!(
                pb::Op::decode(Cursor::new(&$serialized.bytes))
                    .unwrap()
                    .inputs
                    .into_iter()
                    .map(|input| (input.digest, input.index))
                    .collect::<Vec<_>>(),
                $value
                    .into_iter()
                    .map(|input: (&str, i64)| (String::from(input.0), input.1))
                    .collect::<Vec<_>>()
            );
        }};

        ($serialized:expr, $context:expr, cached_tail, $value:expr) => {
            assert_eq!(
                $context
                    .registered_nodes_iter()
                    .map(|node| node.digest.clone())
                    .collect::<Vec<_>>(),
                crate::utils::test::to_vec($value),
            );
        };

        ($serialized:expr, $context:expr, caps, $value:expr) => {{
            let mut caps = $serialized
                .metadata
                .caps
                .into_iter()
                .map(|pair| pair.0)
                .collect::<Vec<_>>();

            caps.sort();
            assert_eq!(caps, crate::utils::test::to_vec($value));
        }};

        ($serialized:expr, $context:expr, description, $value:expr) => {
            assert_eq!(
                $serialized.metadata.description,
                crate::utils::test::to_map($value),
            );
        };

        ($serialized:expr, $context:expr, digest, $value:expr) => {
            assert_eq!($serialized.digest, $value);
        };
    }

    use std::collections::HashMap;

    pub fn to_map(pairs: Vec<(&str, &str)>) -> HashMap<String, String> {
        pairs
            .into_iter()
            .map(|(key, value): (&str, &str)| (key.into(), value.into()))
            .collect()
    }

    pub fn to_vec(items: Vec<&str>) -> Vec<String> {
        items.into_iter().map(String::from).collect()
    }
}
