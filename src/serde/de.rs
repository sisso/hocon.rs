use serde;

use super::error::{Error, Result};
use crate::Hocon;

macro_rules! impl_deserialize_n {
    ($method:ident, $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: serde::de::Visitor<'de>,
        {
            visitor.$visit(
                self.read
                    .get_attribute_value(&self.current_field)
                    .ok_or_else(|| Error { message: format!("missing integer for field {:?}",
                                                            &self.current_field) })?
                    .clone()
                    .as_i64()
                    .ok_or_else(|| Error { message: format!("missing integer for field {:?}",
                                                            &self.current_field) })?
            )
        }
    };
    ($type:ty, $method:ident, $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: serde::de::Visitor<'de>,
        {
            visitor.$visit(
                self.read
                    .get_attribute_value(&self.current_field)
                    .ok_or_else(|| Error { message: format!("missing integer for field {:?}",
                                                            &self.current_field) })?
                    .clone()
                    .as_i64()
                    .ok_or_else(|| Error { message: format!("missing integer for field {:?}",
                                                            &self.current_field) })? as $type
            )
        }
    };
}
macro_rules! impl_deserialize_f {
    ($method:ident, $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: serde::de::Visitor<'de>,
        {
            visitor.$visit(
                self.read
                    .get_attribute_value(&self.current_field)
                    .ok_or_else(|| Error { message: format!("missing float for field {:?}",
                                                            &self.current_field) })?
                    .clone()
                    .as_f64()
                    .ok_or_else(|| Error { message: format!("missing float for field {:?}",
                                                            &self.current_field) })?
            )
        }

    };
    ($type:ty, $method:ident, $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: serde::de::Visitor<'de>,
        {
            visitor.$visit(
                self.read
                    .get_attribute_value(&self.current_field)
                    .ok_or_else(|| Error { message: format!("missing float for field {:?}",
                                                            &self.current_field) })?
                    .clone()
                    .as_f64()
                    .ok_or_else(|| Error { message: format!("missing float for field {:?}",
                                                            &self.current_field) })? as $type
            )
        }
    };
}

#[derive(Debug)]
enum Index {
    String(String),
    Number(usize),
    None,
}

trait Read {
    fn get_attribute_value(&self, index: &Index) -> Option<&Hocon>;
}
struct HoconRead {
    hocon: Hocon,
}
impl HoconRead {
    fn new(hocon: Hocon) -> Self {
        HoconRead { hocon }
    }
}
impl Read for HoconRead {
    fn get_attribute_value(&self, index: &Index) -> Option<&Hocon> {
        match *index {
            Index::String(ref key) => match &self.hocon[key.as_ref()] {
                Hocon::BadValue => None,
                v => Some(v),
            },

            _ => None,
        }
    }
}

struct VecRead {
    vec: Vec<Hocon>,
}

impl Read for VecRead {
    fn get_attribute_value(&self, index: &Index) -> Option<&Hocon> {
        match *index {
            Index::Number(key) => self.vec.get(key),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Deserializer<R> {
    read: R,
    current_field: Index,
    as_key: bool,
}
impl<'de, R> Deserializer<R>
where
    R: Read,
{
    pub fn new(read: R) -> Self {
        Deserializer {
            read,
            current_field: Index::None,
            as_key: false,
        }
    }
}

impl<'de, 'a, R: Read> serde::de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bool(
            self.read
                .get_attribute_value(&self.current_field)
                .ok_or_else(|| Error {
                    message: "Missing field".to_owned(),
                })?
                .clone()
                .as_bool()
                .ok_or_else(|| Error {
                    message: "Invalid type".to_owned(),
                })?,
        )
    }

    impl_deserialize_n!(i8, deserialize_i8, visit_i8);
    impl_deserialize_n!(i16, deserialize_i16, visit_i16);
    impl_deserialize_n!(i32, deserialize_i32, visit_i32);
    impl_deserialize_n!(deserialize_i64, visit_i64);
    // impl_deserialize_n!(i64, deserialize_i64, visit_i64);

    impl_deserialize_n!(u8, deserialize_u8, visit_u8);
    impl_deserialize_n!(u16, deserialize_u16, visit_u16);
    impl_deserialize_n!(u32, deserialize_u32, visit_u32);
    impl_deserialize_n!(u64, deserialize_u64, visit_u64);

    impl_deserialize_f!(f32, deserialize_f32, visit_f32);
    impl_deserialize_f!(deserialize_f64, visit_f64);
    // impl_deserialize_f!(f64, deserialize_f64, visit_f64);

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_char(
            self.read
                .get_attribute_value(&self.current_field)
                .ok_or_else(|| Error {
                    message: format!("missing char for field {:?}", &self.current_field),
                })?
                .clone()
                .as_string()
                .ok_or_else(|| Error {
                    message: format!("missing char for field {:?}", &self.current_field),
                })?
                .parse::<char>()
                .map_err(|_| Error {
                    message: "Invalid type".to_owned(),
                })?,
        )
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.as_key {
            match &self.current_field {
                Index::String(ref key) => visitor.visit_str(key),
                _ => visitor.visit_str(""),
            }
        } else if let Some(field) = self.read.get_attribute_value(&self.current_field) {
            field
                .clone()
                .as_string()
                .ok_or_else(|| Error {
                    message: format!("missing string for field {:?}", &self.current_field),
                })
                .and_then(|string_field| visitor.visit_str(&string_field))
        } else {
            visitor.visit_str("")
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.read.get_attribute_value(&self.current_field).is_none() {
            return visitor.visit_none();
        }
        match self
            .read
            .get_attribute_value(&self.current_field)
            .ok_or_else(|| Error {
                message: format!("missing option for field {:?}", &self.current_field),
            })? {
            Hocon::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let list = self
            .read
            .get_attribute_value(&self.current_field)
            .ok_or_else(|| Error {
                message: format!("missing sequence for field {:?}", &self.current_field),
            })?
            .clone();
        let read = if let Hocon::Array(list) = list {
            VecRead { vec: list }
        } else {
            return Err(Error {
                message: "No sequence input found".to_owned(),
            });
        };
        let mut des = Deserializer::new(read);
        visitor.visit_seq(SeqAccess::new(&mut des))
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current_field {
            Index::None => visitor.visit_map(MapAccess::new(self, fields)),
            _ => {
                let hc = self
                    .read
                    .get_attribute_value(&self.current_field)
                    .ok_or_else(|| Error {
                        message: format!("missing struct for field {:?}", &self.current_field),
                    })?
                    .clone();
                let mut des = Deserializer::new(HoconRead::new(hc));
                visitor.visit_map(MapAccess::new(&mut des, fields))
            }
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current_field {
            Index::String(ref value) => visitor.visit_str(&value.clone()),
            _ => Err(Error {
                message: "indentifier should be a string".to_string(),
            }),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct SeqAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
    current: usize,
}

impl<'a, R: 'a> SeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        SeqAccess { de, current: 0 }
    }
}

impl<'de, 'a, R: Read + 'a> serde::de::SeqAccess<'de> for SeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        self.de.current_field = Index::Number(self.current);
        self.current += 1;
        if self
            .de
            .read
            .get_attribute_value(&self.de.current_field)
            .is_none()
        {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct MapAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
    keys: &'static [&'static str],
    current: usize,
}

impl<'a, R: 'a> MapAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, keys: &'static [&'static str]) -> Self {
        MapAccess {
            de,
            keys,
            current: 0,
        }
    }
}

impl<'de, 'a, R: Read + 'a> serde::de::MapAccess<'de> for MapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.current >= self.keys.len() {
            Ok(None)
        } else {
            self.de.current_field = Index::String(self.keys[self.current].to_string());
            self.de.as_key = true;
            self.current += 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        self.de.as_key = false;
        seed.deserialize(&mut *self.de)
    }
}

fn from_trait<'de, R, T>(read: R) -> Result<T>
where
    R: Read,
    T: serde::de::Deserialize<'de>,
{
    let mut de = Deserializer::new(read);
    let value = serde::de::Deserialize::deserialize(&mut de)?;

    Ok(value)
}

/// Deserialize an instance of type `T` from an HOCON document at `file_path`
pub fn from_file_path<'a, T>(file_path: &str) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    from_trait(HoconRead::new(Hocon::load_from_file(file_path).map_err(
        |_| Error {
            message: format!("Couldn't parse file '{}' as a HOCON document", file_path),
        },
    )?))
}

/// Deserialize an instance of type `T` from an HOCON document in `str`
pub fn from_str<'a, T>(hocon: &str) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    from_trait(HoconRead::new(Hocon::load_from_str(hocon).map_err(
        |_| Error {
            message: format!("Couldn't parse '{}' as a HOCON document", hocon),
        },
    )?))
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    #[test]
    fn can_deserialize_struct() {
        #[derive(Deserialize, Debug)]
        struct Internal {
            a: i64,
            b: f64,
            c: Option<u64>,
        }
        #[derive(Deserialize, Debug)]
        struct Basic {
            intern: Vec<Internal>,
            d: i32,
            e: f32,
            f: bool,
            g: String,
        }

        let doc = r#"{d:56, e:543.12, f:false, g: test, intern:[
            {a:8,b:1.5,c:1919},
            {a:8,b:0},
            {a:1,b:2,c:null},
]}"#;

        let res: super::Result<Basic> = super::from_str(doc);
        assert!(res.is_ok());
    }
}