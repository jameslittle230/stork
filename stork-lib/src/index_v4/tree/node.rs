use std::collections::{BTreeMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use super::arena::ArenaIndex;

/// The edges of our tree are indexable by a char, instead of by a number or by
/// left/right.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Node<U>
where
    U: Debug + Clone + Hash + Eq,
{
    value: HashSet<U>,
    children: BTreeMap<char, ArenaIndex>,
}

impl<U> PartialEq for Node<U>
where
    U: Debug + Clone + Hash + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.children == other.children
    }
}

// impl<U> PartialOrd for Node<U>
// where
//     U: Debug + Clone + Hash + Eq,
// {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         match self.value.partial_cmp(&other.value) {
//             Some(core::cmp::Ordering::Equal) => {}
//             ord => return ord,
//         }
//         self.children.partial_cmp(&other.children)
//     }
// }

impl<U> Node<U>
where
    U: Debug + Clone + Hash + Eq,
{
    pub(crate) fn new() -> Self {
        Self {
            value: HashSet::new(),
            children: BTreeMap::new(),
        }
    }
    pub(crate) fn get_child(&self, key: &char) -> Option<&ArenaIndex> {
        self.children.get(key)
    }

    pub(crate) fn push_child(&mut self, key: char, child_index: ArenaIndex) {
        self.children.insert(key, child_index);
    }

    pub(crate) fn set_value(&mut self, value: U) {
        self.value.insert(value);
    }

    pub(crate) fn get_values(&self) -> Vec<U> {
        self.value.clone().into_iter().collect::<Vec<U>>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub(super) struct NodeChild {
    pub(super) categorization: NodeValueCategorization,
    pub(super) arena_index: ArenaIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub(super) enum NodeValueCategorization {
    Exact,
    Prefix,
}
