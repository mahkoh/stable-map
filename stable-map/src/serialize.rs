#[cfg(test)]
mod tests;

use {
    crate::StableMap,
    core::{
        fmt::Formatter,
        hash::{BuildHasher, Hash},
    },
    serde::{
        de::{MapAccess, Visitor},
        ser::SerializeMap,
        Deserialize, Deserializer, Serialize, Serializer,
    },
};

impl<K, V, H> Serialize for StableMap<K, V, H>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (key, value) in self {
            map.serialize_entry(&key, &value)?;
        }
        map.end()
    }
}

impl<'de, K, V, S> Deserialize<'de> for StableMap<K, V, S>
where
    K: Eq + Hash + Deserialize<'de>,
    V: Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(Vis(StableMap::default()))
    }
}

struct Vis<K, V, S>(StableMap<K, V, S>);

impl<'de, K, V, S> Visitor<'de> for Vis<K, V, S>
where
    K: Eq + Hash + Deserialize<'de>,
    V: Deserialize<'de>,
    S: BuildHasher,
{
    type Value = StableMap<K, V, S>;

    fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
        write!(formatter, "a map")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some((key, value)) = map.next_entry()? {
            self.0.insert(key, value);
        }
        Ok(self.0)
    }
}
