use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

pub(crate) fn ordered_map<K: Ord + Serialize, V: Serialize, S>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
