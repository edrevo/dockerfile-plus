use std::collections::HashMap;

use buildkit_proto::moby::buildkit::v1::frontend::CacheOptionsEntry as CacheOptionsEntryProto;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CacheOptionsEntry {
    #[serde(rename = "Type")]
    pub cache_type: CacheType,

    #[serde(rename = "Attrs")]
    pub attrs: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    Local,
    Registry,
    Inline,
}

impl Into<CacheOptionsEntryProto> for CacheOptionsEntry {
    fn into(self) -> CacheOptionsEntryProto {
        CacheOptionsEntryProto {
            r#type: self.cache_type.into(),
            attrs: self.attrs,
        }
    }
}

impl Into<String> for CacheType {
    fn into(self) -> String {
        match self {
            CacheType::Local => "local".into(),
            CacheType::Registry => "registry".into(),
            CacheType::Inline => "inline".into(),
        }
    }
}
