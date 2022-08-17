mod arena;
mod node;

use std::{collections::BTreeMap, fmt::Debug, hash::Hash};

use serde::{Deserialize, Serialize};

use arena::Arena;
use node::Node;

pub trait NodeValueTrait: Debug + Clone + Hash + Eq + Ord {}

impl<T> NodeValueTrait for T where T: Debug + Clone + Hash + Eq + Ord {}

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
    U: NodeValueTrait,
{
    min_char_insertion_length: usize,
    arena: Arena<Node<U>>,
}

impl<U> Default for Tree<U>
where
    U: NodeValueTrait,
{
    fn default() -> Self {
        Tree {
            min_char_insertion_length: 2,
            arena: Arena::new(Node::new()),
        }
    }
}

impl<U> Tree<U>
where
    U: NodeValueTrait,
{
    pub(crate) fn push_value_for_string(&mut self, word: &str, value: U) {
        let mut current_index = self.arena.root.unwrap();

        // Traverse down the tree, per character
        let word_len = word.len();
        for (idx, char) in word.chars().enumerate() {
            // word_len - idx, with u8-based safety
            let chars_remaining = word_len.try_into().unwrap_or(u8::MAX)
                - idx.try_into().unwrap_or(u8::max_value())
                - 1; // TODO: Make sure unchecked subtraction isn't going to cause panics
            let current_node = self.arena.node_at(current_index).unwrap();

            match current_node.get_child(&char).cloned() {
                Some(next_index) => {
                    let next_node = self.arena.node_at_mut(next_index).unwrap();
                    if idx >= self.min_char_insertion_length {
                        next_node.set_value(value.clone(), chars_remaining);
                    }
                    current_index = next_index;
                }
                None => {
                    let mut new_node: Node<U> = Node::new();
                    if idx >= self.min_char_insertion_length {
                        new_node.set_value(value.clone(), chars_remaining);
                    }
                    let next_index = self.arena.add_node(new_node);
                    self.arena
                        .node_at_mut(current_index)
                        .unwrap()
                        .push_child(char, next_index);
                    current_index = next_index;
                }
            };
        }
        let node = self.arena.node_at_mut(current_index).unwrap();
        node.set_value(value, 0);
    }

    pub(crate) fn get_values_for_string(&self, word: &str) -> Option<Vec<(u8, U)>> {
        if let Some(curr) = self.arena.root.as_ref() {
            let mut curr = curr;
            for char in word.chars() {
                match self
                    .arena
                    .node_at(curr.to_owned())
                    .and_then(|node| node.get_child(&char))
                {
                    Some(new_node) => curr = new_node,
                    None => return None,
                }
            }

            self.arena
                .node_at(curr.to_owned())
                .map(|node| node.get_values())
                .map(|categorized_values| {
                    categorized_values
                        .iter()
                        .map(|nv| (nv.chars_remaining, nv.value.to_owned())) // TODO: Can I rm these hot-path clones?
                        .collect::<Vec<(u8, U)>>()
                })
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
            tree.get_values_for_string("tes").map(|mut vec| {
                vec.sort_by_key(|tuple| tuple.1);
                vec
            }),
            Some(vec![(6, 2), (1, 1)]).map(|mut vec| {
                vec.sort_by_key(|tuple| tuple.1);
                vec
            })
        );

        assert_eq!(tree.get_values_for_string("tess"), Some(vec![(5, 2)]));
        assert_eq!(tree.get_values_for_string("test"), Some(vec![(0, 1)]));
        assert_eq!(tree.get_values_for_string("tesseract"), Some(vec![(0, 2)]));
    }

    #[test]
    fn returns_none_when_no_result() {
        let mut tree = Tree::default();
        tree.push_value_for_string("hyperbolic", 42);
        assert_eq!(tree.get_values_for_string("tower"), None);
    }
}
