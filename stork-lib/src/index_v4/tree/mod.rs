mod arena;
mod node;

use std::{
    collections::{BTreeMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use serde::{Deserialize, Serialize};

use arena::Arena;
use node::Node;

/// A tree data structure (backed by an Arena) where each node's value is a
/// `HashSet` of generic U values.
///
/// Uniquely, a node's children are indexable by a char, rather than by hardcoded
/// values (e.g. left/right) or by an index.
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
/// To use this data structure, push values of type U into the tree, keyed
/// by a string. The value will be available at each string subset.
///
///
/// ```
/// let mut tree = Tree::default();
/// tree.push_value_for_string("test", 1);
///
/// assert_eq!(tree.get_value_for_string("t").is_empty(), true);
/// assert_eq!(tree.get_value_for_string("te").is_empty(), true);
/// assert_eq!(tree.get_value_for_string("tes"), vec![1]);
/// assert_eq!(tree.get_value_for_string("test"), vec![1]);
/// ```
///
/// It is unsupported to update values that have already been pushed into the
/// tree.
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Tree<U>
where
    U: Debug + Clone + Hash + Eq,
{
    min_char_insertion_length: usize,
    arena: Arena<Node<HashSet<U>>>,
}

impl<U> Default for Tree<U>
where
    U: Debug + Clone + Hash + Eq,
{
    fn default() -> Self {
        Tree {
            min_char_insertion_length: 2,
            arena: Arena::new(Tree::build_node()),
        }
    }
}

impl<U> Tree<U>
where
    U: Debug + Clone + Hash + Eq,
{
    fn build_node() -> Node<HashSet<U>> {
        Node {
            value: HashSet::new(),
            children: BTreeMap::default(),
        }
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
                if count >= self.min_char_insertion_length {
                    if let Some(node) = self.arena.node_at_mut(current_index.to_owned()) {
                        node.value.insert(value.clone());
                    }
                }
            } else {
                let mut new_node: Node<HashSet<U>> = Tree::build_node();
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

    pub(crate) fn get_values_for_string(&self, word: &str) -> Option<Vec<U>> {
        if let Some(curr) = self.arena.root.as_ref() {
            let mut curr = curr;
            for char in word.chars() {
                match self
                    .arena
                    .node_at(curr.to_owned())
                    .and_then(|node| node.children.get(&char))
                {
                    Some(new_node) => curr = new_node,
                    None => return None,
                }
            }

            self.arena
                .node_at(curr.to_owned())
                .map(|node| node.value.clone().into_iter().collect::<Vec<U>>())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn insert_values_two_words() {
        let mut tree = Tree::default();
        tree.push_value_for_string("test", 1);
        tree.push_value_for_string("tesseract", 2);

        assert_eq!(tree.get_values_for_string("t").unwrap().is_empty(), true);
        assert_eq!(tree.get_values_for_string("te").unwrap().is_empty(), true);
        assert_eq!(
            tree.get_values_for_string("tes").map(|mut v| {
                v.sort_unstable();
                v
            }),
            Some(vec![1, 2])
        );
        assert_eq!(tree.get_values_for_string("tess"), Some(vec![2]));
        assert_eq!(tree.get_values_for_string("test"), Some(vec![1]));
        assert_eq!(tree.get_values_for_string("tesseract"), Some(vec![2]));
    }

    #[test]
    fn returns_none_when_no_result() {
        let mut tree = Tree::default();
        tree.push_value_for_string("hyperbolic", 42);
        assert_eq!(tree.get_values_for_string("tower"), None);
    }
}
