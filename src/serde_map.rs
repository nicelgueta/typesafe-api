//! (De)serializes `Vec<(String, V)>` as a JSON object, preserving insertion
//! order instead of the sorted order a `BTreeMap` would impose.

use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

pub fn serialize<S, V>(entries: &[(String, V)], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Serialize,
{
    serializer.collect_map(entries.iter().map(|(k, v)| (k, v)))
}

pub fn deserialize<'de, D, V>(deserializer: D) -> Result<Vec<(String, V)>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct MapVisitor<V>(PhantomData<V>);

    impl<'de, V> Visitor<'de> for MapVisitor<V>
    where
        V: Deserialize<'de>,
    {
        type Value = Vec<(String, V)>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));
            while let Some(entry) = map.next_entry()? {
                entries.push(entry);
            }
            Ok(entries)
        }
    }

    deserializer.deserialize_map(MapVisitor(PhantomData))
}
