use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use super::arena::ArenaIndex;

/// The edges of our tree are indexable by a char, instead of by a number or by
/// left/right.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct Node<U>
where
    U: Debug + Clone,
{
    pub(crate) value: U,
    pub(crate) children: BTreeMap<char, ArenaIndex>,
}

impl<U> Node<U>
where
    U: Debug + Clone,
{
    pub(crate) fn from_value(value: U) -> Self {
        Node {
            value,
            children: BTreeMap::new(),
        }
    }

    pub(crate) fn push_child(&mut self, key: char, child_index: ArenaIndex) {
        self.children.insert(key, child_index);
    }
}
