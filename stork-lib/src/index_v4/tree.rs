use std::{
    collections::{BTreeMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use serde::{Deserialize, Serialize};

type ArenaIndex = usize;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Arena<T> {
    values: Vec<Option<T>>,
    root: Option<ArenaIndex>,
}

impl<T> Arena<T> {
    fn set_root(&mut self, root: Option<ArenaIndex>) {
        self.root = root;
    }

    fn add_node(&mut self, node: T) -> ArenaIndex {
        let index = self.values.len();
        self.values.push(Some(node));
        index
    }

    fn remove_node_at(&mut self, index: ArenaIndex) -> Option<T> {
        if let Some(node) = self.values.get_mut(index) {
            node.take()
        } else {
            None
        }
    }

    fn node_at(&self, index: ArenaIndex) -> Option<&T> {
        return if let Some(node) = self.values.get(index) {
            node.as_ref()
        } else {
            None
        };
    }

    fn node_at_mut(&mut self, index: ArenaIndex) -> Option<&mut T> {
        return if let Some(node) = self.values.get_mut(index) {
            node.as_mut()
        } else {
            None
        };
    }
}

/**
 * The edges of our tree are indexable by a char, instead of by a number or by
 * left/right.
 */
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct CharEdgeNode<U>
where
    U: Debug + Clone,
{
    pub(crate) value: U,
    pub(crate) children: BTreeMap<char, ArenaIndex>,
}

impl<U> CharEdgeNode<U>
where
    U: Debug + Clone,
{
    pub(crate) fn from_value(value: U) -> Self {
        CharEdgeNode {
            value,
            children: BTreeMap::new(),
        }
    }

    pub(crate) fn push_child(&mut self, key: char, child_index: ArenaIndex) {
        self.children.insert(key, child_index);
    }
}

/// A tree data structure (backed by an Arena) where each node's value is a
/// `HashSet` of U values, and where a node's children are represented by a char.
///
/// ```txt
///                         +--------+                        
///                         |  ROOT  |                        
///                         +--------+                        
///                              |                            
///       +-----------+----------+-----------+-----------+    
///      a|          b|         c|          d|          e|    
///  +--------+  +--------+ +--------+  +--------+  +--------+
///  | [1, 2] |  |  [ ]   | |  [2]   |  | [2, 3] |  | [5, 6] |
///  +--------+  +--------+ +--------+  +--------+  +--------+
///                                          |                
///       +-----------+----------+-----------+-----------+    
///      a|          b|         c|          d|          e|    
///  +--------+  +--------+ +--------+  +--------+  +--------+
///  | [1, 2] |  |  [ ]   | |  [2]   |  | [2, 3] |  | [5, 6] |
///  +--------+  +--------+ +--------+  +--------+  +--------+
/// ```
///
/// Usage is sort of like a hash map. You insert a key/value pair and the value
/// gets inserted into the node at each character up through the spelling of that word.
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CharEdgeSetTree<U>
where
    U: Debug + Clone + Hash + Eq,
{
    arena: Arena<CharEdgeNode<HashSet<U>>>,
}

impl<U> Default for CharEdgeSetTree<U>
where
    U: Debug + Clone + Hash + Eq,
{
    fn default() -> Self {
        CharEdgeSetTree {
            arena: Arena {
                values: vec![Some(CharEdgeSetTree::build_node())],
                root: Some(0),
            },
        }
    }
}

impl<U> CharEdgeSetTree<U>
where
    U: Debug + Clone + Hash + Eq,
{
    fn build_node() -> CharEdgeNode<HashSet<U>> {
        CharEdgeNode {
            value: HashSet::new(),
            children: BTreeMap::default(),
        }
    }

    fn node_at(&self, index: usize) -> Option<&CharEdgeNode<HashSet<U>>> {
        self.arena.node_at(index)
    }

    pub(crate) fn push_value_for_string(&mut self, word: &str, value: U) {
        let mut current_index = self.arena.root.unwrap();

        for (count, char) in word.chars().enumerate() {
            let next_index = self
                .arena
                .node_at(current_index)
                .unwrap()
                .children
                .get(&char);

            if let Some(next_index) = next_index {
                current_index = next_index.to_owned();
                if count >= 2 {
                    if let Some(node) = self.arena.node_at_mut(current_index.to_owned()) {
                        node.value.insert(value.clone());
                    }
                }
            } else {
                let mut new_node: CharEdgeNode<HashSet<U>> = CharEdgeSetTree::build_node();
                if count >= 2 {
                    new_node.value.insert(value.clone());
                }
                let next_index = self.arena.add_node(new_node);
                self.arena
                    .node_at_mut(current_index)
                    .unwrap()
                    .children
                    .insert(char, next_index);
                current_index = next_index;
            }
        }
        let node = self.arena.node_at_mut(current_index).unwrap();
        node.value.insert(value);
    }

    pub(crate) fn get_value_for_string(&self, word: &str) -> Vec<U> {
        let mut curr = self.arena.root.as_ref();
        for char in word.chars() {
            curr = self
                .arena
                .node_at(curr.unwrap().to_owned())
                .unwrap()
                .children
                .get(&char);
        }

        self.arena
            .node_at(curr.unwrap().to_owned())
            .unwrap()
            .value
            .clone()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn insert_values() {
        let mut tree = CharEdgeSetTree::default();
        tree.push_value_for_string("test", 1);

        assert_eq!(tree.get_value_for_string("t").is_empty(), true);
        assert_eq!(tree.get_value_for_string("te").is_empty(), true);
        assert_eq!(tree.get_value_for_string("tes"), vec![1]);
        assert_eq!(tree.get_value_for_string("test"), vec![1]);
    }

    #[test]
    fn insert_values_two_words() {
        let mut tree = CharEdgeSetTree::default();
        tree.push_value_for_string("test", 1);
        tree.push_value_for_string("tesseract", 2);

        assert_eq!(tree.get_value_for_string("t").is_empty(), true);
        assert_eq!(tree.get_value_for_string("te").is_empty(), true);
        assert_eq!(tree.get_value_for_string("tes"), vec![1, 2]);
        assert_eq!(tree.get_value_for_string("tess"), vec![2]);
        assert_eq!(tree.get_value_for_string("test"), vec![1]);
        assert_eq!(tree.get_value_for_string("tesseract"), vec![2]);
    }
}
