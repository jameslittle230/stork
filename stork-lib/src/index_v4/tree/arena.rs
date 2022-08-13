use serde::{Deserialize, Serialize};

pub(crate) type ArenaIndex = usize;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct Arena<T> {
    values: Vec<Option<T>>,
    pub(crate) root: Option<ArenaIndex>,
}

impl<T> Arena<T> {
    pub(crate) fn new(node: T) -> Self {
        Self {
            values: vec![Some(node)],
            root: Some(0),
        }
    }

    pub(crate) fn set_root(&mut self, root: Option<ArenaIndex>) {
        self.root = root;
    }

    pub(crate) fn add_node(&mut self, node: T) -> ArenaIndex {
        let index = self.values.len();
        self.values.push(Some(node));
        index
    }

    pub(crate) fn remove_node_at(&mut self, index: ArenaIndex) -> Option<T> {
        if let Some(node) = self.values.get_mut(index) {
            node.take()
        } else {
            None
        }
    }

    pub(crate) fn node_at(&self, index: ArenaIndex) -> Option<&T> {
        return if let Some(node) = self.values.get(index) {
            node.as_ref()
        } else {
            None
        };
    }

    pub(crate) fn node_at_mut(&mut self, index: ArenaIndex) -> Option<&mut T> {
        return if let Some(node) = self.values.get_mut(index) {
            node.as_mut()
        } else {
            None
        };
    }
}
