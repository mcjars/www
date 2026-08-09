use serde::{
    Deserialize, Deserializer,
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
use std::cell::Cell;

pub const MAX_NODE_DEPTH: usize = 128;

pub struct NodeBudget<'a> {
    nodes: &'a Cell<usize>,
    depth: usize,
}

impl<'a> NodeBudget<'a> {
    #[inline]
    pub fn new(nodes: &'a Cell<usize>) -> Self {
        Self { nodes, depth: 0 }
    }

    #[inline]
    fn spend<E: de::Error>(&self) -> Result<(), E> {
        match self.nodes.get().checked_sub(1) {
            Some(nodes) => {
                self.nodes.set(nodes);

                Ok(())
            }
            None => Err(E::custom("config expands to too many values")),
        }
    }

    #[inline]
    fn nested<E: de::Error>(&self) -> Result<Self, E> {
        if self.depth >= MAX_NODE_DEPTH {
            return Err(E::custom("config is nested too deeply"));
        }

        Ok(Self {
            nodes: self.nodes,
            depth: self.depth + 1,
        })
    }
}

impl<'de> DeserializeSeed<'de> for NodeBudget<'_> {
    type Value = ();

    #[inline]
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

macro_rules! visit_scalar {
    ($($name:ident($type:ty),)*) => {$(
        #[inline]
        fn $name<E: de::Error>(self, _value: $type) -> Result<(), E> {
            self.spend()
        }
    )*};
}

impl<'de> Visitor<'de> for NodeBudget<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any value")
    }

    visit_scalar! {
        visit_bool(bool),
        visit_i64(i64),
        visit_i128(i128),
        visit_u64(u64),
        visit_u128(u128),
        visit_f64(f64),
        visit_str(&str),
        visit_bytes(&[u8]),
    }

    #[inline]
    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        self.spend()
    }

    #[inline]
    fn visit_none<E: de::Error>(self) -> Result<(), E> {
        self.spend()
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        self.spend::<D::Error>()?;

        deserializer.deserialize_any(self.nested::<D::Error>()?)
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self.nested::<D::Error>()?)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        self.spend::<A::Error>()?;

        while seq.next_element_seed(self.nested::<A::Error>()?)?.is_some() {}

        Ok(())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        self.spend::<A::Error>()?;

        while map.next_key_seed(self.nested::<A::Error>()?)?.is_some() {
            map.next_value_seed(self.nested::<A::Error>()?)?;
        }

        Ok(())
    }

    fn visit_enum<A: de::EnumAccess<'de>>(self, data: A) -> Result<(), A::Error> {
        use de::VariantAccess;

        self.spend::<A::Error>()?;

        let (_, variant) = data.variant_seed(self.nested::<A::Error>()?)?;

        variant.newtype_variant_seed(self.nested::<A::Error>()?)
    }
}

pub fn deserialize_string_option<'de, D>(
    deserializer: D,
) -> Result<Option<compact_str::CompactString>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<compact_str::CompactString> =
        Option::deserialize(deserializer).unwrap_or_default();
    Ok(value.filter(|s| !s.is_empty()))
}
