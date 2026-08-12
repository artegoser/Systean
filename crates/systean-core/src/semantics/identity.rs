use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! semantic_id {
    ($name:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Deserialize,
            Serialize,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
        )]
        pub struct $name(pub u64);

        impl $name {
            pub fn from_source(namespace: &str, name: &str) -> Self {
                Self(stable_identity(namespace, name))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:016x}", self.0)
            }
        }
    };
}

semantic_id!(SymbolId);
semantic_id!(TypeId);
semantic_id!(ConstructorId);
semantic_id!(ContextSlotId);
semantic_id!(DimensionId);
semantic_id!(UnitId);
semantic_id!(IntrinsicId);
semantic_id!(FieldId);

fn stable_identity(namespace: &str, name: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in namespace
        .as_bytes()
        .iter()
        .copied()
        .chain([0])
        .chain(name.as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
