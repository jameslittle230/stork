use std::fmt::Debug;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::Iter,
};

use serde::{Deserialize, Serialize};

use super::arena::ArenaIndex;

/// The edges of our tree are indexable by a char, instead of by a number or by
/// left/right.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct Node<U>
where
    U: super::NodeValueTrait,
{
    values: HashSet<U>,
    children: BTreeMap<char, ArenaIndex>,
}

impl<U> Node<U>
where
    U: super::NodeValueTrait,
{
    fn sorted_value(&self) -> Vec<&U> {
        let mut vec = self.values.iter().collect::<Vec<&U>>();
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

impl<U> Node<U>
where
    U: super::NodeValueTrait,
{
    pub(super) fn new() -> Self {
        Self {
            values: HashSet::new(),
            children: BTreeMap::new(),
        }
    }
    pub(super) fn get_child(&self, key: &char) -> Option<&ArenaIndex> {
        self.children.get(key)
    }

    pub(super) fn push_child(&mut self, key: char, child_index: ArenaIndex) {
        self.children.insert(key, child_index);
    }

    pub(super) fn set_value(&mut self, value: U) {
        self.values.replace(value);
    }

    pub(super) fn get_values(&self) -> Vec<U> {
        self.values.clone().into_iter().collect()
    }

    pub(super) fn get_all_children(&self) -> Vec<ArenaIndex> {
        return self.children.values().cloned().collect();
    }
}
