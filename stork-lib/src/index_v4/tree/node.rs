use std::collections::{BTreeMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use super::arena::ArenaIndex;

/// The edges of our tree are indexable by a char, instead of by a number or by
/// left/right.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct Node<U>
where
    U: super::NodeValueTrait,
{
    value: HashSet<NodeValue<U>>,
    children: BTreeMap<char, ArenaIndex>,
}

impl<U> Node<U>
where
    U: super::NodeValueTrait,
{
    fn sorted_value(&self) -> Vec<&NodeValue<U>> {
        let mut vec = self.value.iter().collect::<Vec<&NodeValue<U>>>();
        vec.sort();
        vec
    }
}

impl<U> PartialEq for Node<U>
where
    U: super::NodeValueTrait,
{
    fn eq(&self, other: &Self) -> bool {
        self.sorted_value() == other.sorted_value() && self.children == other.children
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
    U: super::NodeValueTrait,
{
    pub(super) fn new() -> Self {
        Self {
            value: HashSet::new(),
            children: BTreeMap::new(),
        }
    }
    pub(super) fn get_child(&self, key: &char) -> Option<&ArenaIndex> {
        self.children.get(key)
    }

    pub(super) fn push_child(&mut self, key: char, child_index: ArenaIndex) {
        self.children.insert(key, child_index);
    }

    pub(super) fn set_value(&mut self, value: U, chars_remaining: u8) {
        self.value.replace(NodeValue {
            chars_remaining,
            value,
        });
    }

    pub(super) fn get_values(&self) -> Vec<NodeValue<U>> {
        self.value
            .clone()
            .into_iter()
            .collect::<Vec<NodeValue<U>>>()
    }
}

#[derive(Debug, Clone, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(super) struct NodeValue<U>
where
    U: super::NodeValueTrait,
{
    pub(super) chars_remaining: u8,
    pub(super) value: U,
}

impl<U> PartialEq for NodeValue<U>
where
    U: super::NodeValueTrait,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<U> Hash for NodeValue<U>
where
    U: super::NodeValueTrait,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
