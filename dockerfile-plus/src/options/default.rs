use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
enum OptionValue {
    Flag(bool),
    Single(String),
    Multiple(Vec<String>),
}
