use std::io::Cursor;
use std::iter::empty;

use anyhow::Result;
use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::de::{self, DeserializeOwned, IntoDeserializer, Visitor};
use serde::forward_to_deserialize_any;

pub fn from_env<T, I>(pairs: I) -> Result<T>
where
    T: DeserializeOwned,
    I: IntoIterator<Item = (String, String)>,
{
    let owned_pairs = pairs.into_iter().collect::<Vec<_>>();
    let pairs = {
        owned_pairs.iter().filter_map(|(name, value)| {
            if name.starts_with("BUILDKIT_FRONTEND_OPT_") {
                Some(value)
            } else {
                None
            }
        })
    };

    let deserializer = EnvDeserializer {
        vals: pairs.map(|value| extract_name_and_value(&value)),
    };

    Ok(T::deserialize(deserializer)?)
}

#[derive(Debug)]
struct EnvDeserializer<P> {
    vals: P,
}

#[derive(Debug)]
enum EnvValue<'de> {
    Flag,
    Json(&'de str),
    Text(&'de str),
}

#[derive(Debug)]
struct EnvItem<'de>(&'de str);

fn extract_name_and_value(mut raw_value: &str) -> (&str, EnvValue) {
    if raw_value.starts_with("build-arg:") {
        raw_value = raw_value.trim_start_matches("build-arg:");
    }

    let mut parts = raw_value.splitn(2, '=');
    let name = parts.next().unwrap();

    match parts.next() {
        None => (name, EnvValue::Flag),
        Some(text) if text.is_empty() => (name, EnvValue::Flag),
        Some(text) if &text[0..1] == "[" || &text[0..1] == "{" => (name, EnvValue::Json(text)),
        Some(text) => (name, EnvValue::Text(text)),
    }
}

impl<'de> IntoDeserializer<'de, serde::de::value::Error> for EnvValue<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> IntoDeserializer<'de, serde::de::value::Error> for EnvItem<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> EnvItem<'de> {
    fn infer<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, serde::de::value::Error> {
        match self.0 {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),

            _ => visitor.visit_str(self.0),
        }
    }

    fn json<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, serde::de::value::Error> {
        use serde::de::Deserializer;
        use serde::de::Error;

        serde_json::Deserializer::from_reader(Cursor::new(self.0))
            .deserialize_any(visitor)
            .map_err(serde::de::value::Error::custom)
    }
}

impl<'de, P> de::Deserializer<'de> for EnvDeserializer<P>
where
    P: Iterator<Item = (&'de str, EnvValue<'de>)>,
{
    type Error = serde::de::value::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(MapDeserializer::new(self.vals))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

// The approach is shamelessly borrowed from https://github.com/softprops/envy/blob/master/src/lib.rs#L113
macro_rules! forward_parsed_values_env_value {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where V: de::Visitor<'de>
            {
                match self {
                    EnvValue::Flag => self.deserialize_any(visitor),
                    EnvValue::Json(_) => self.deserialize_any(visitor),
                    EnvValue::Text(contents) => {
                        match contents.parse::<$ty>() {
                            Ok(val) => val.into_deserializer().$method(visitor),
                            Err(e) => Err(de::Error::custom(format_args!("{} while parsing value '{}'", e, contents)))
                        }
                    }
                }
            }
        )*
    }
}

macro_rules! forward_parsed_values_env_item {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where V: de::Visitor<'de>
            {
                match self.0.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(de::Error::custom(format_args!("{} while parsing value '{}'", e, self.0)))
                }
            }
        )*
    }
}

impl<'de> de::Deserializer<'de> for EnvValue<'de> {
    type Error = serde::de::value::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            EnvValue::Flag => visitor.visit_bool(true),
            EnvValue::Json(contents) => EnvItem(contents).json(visitor),
            EnvValue::Text(contents) => {
                if !contents.contains(',') {
                    EnvItem(contents).infer(visitor)
                } else {
                    SeqDeserializer::new(contents.split(',')).deserialize_seq(visitor)
                }
            }
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            EnvValue::Flag => SeqDeserializer::new(empty::<&'de str>()).deserialize_seq(visitor),
            EnvValue::Json(contents) => EnvItem(contents).json(visitor),
            EnvValue::Text(contents) => {
                SeqDeserializer::new(contents.split(',')).deserialize_seq(visitor)
            }
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_some(self)
    }

    forward_parsed_values_env_value! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        u128 => deserialize_u128,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        i128 => deserialize_i128,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }

    forward_to_deserialize_any! {
        byte_buf
        bytes
        char
        enum
        identifier
        ignored_any
        map
        newtype_struct
        str
        string
        struct
        tuple
        tuple_struct
        unit
        unit_struct
    }
}

impl<'de> de::Deserializer<'de> for EnvItem<'de> {
    type Error = serde::de::value::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.into_deserializer().deserialize_any(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.json(visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.json(visitor)
    }

    forward_parsed_values_env_item! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        u128 => deserialize_u128,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        i128 => deserialize_i128,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }

    forward_to_deserialize_any! {
        byte_buf
        bytes
        char
        enum
        identifier
        ignored_any
        newtype_struct
        option
        seq
        str
        string
        tuple
        tuple_struct
        unit
        unit_struct
    }
}
