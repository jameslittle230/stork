mod arena;
mod node;

use std::{collections::VecDeque, fmt::Debug, hash::Hash};

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
    #[serde(skip)]
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
        let mut current_index = self.arena.root;

        // Traverse down the tree, per character, adding nodes for intermediate
        // characters if they're needed along the way
        let word_len = word.len();
        for (idx, char) in word.chars().enumerate() {
            let current_node = self.arena.node_at(current_index).unwrap();

            match current_node.get_child(&char).cloned() {
                Some(next_index) => {
                    let next_node = &mut self.arena.node_at_mut(next_index).unwrap();
                    current_index = next_index;
                }
                None => {
                    let next_index = self.arena.add_node(Node::new());
                    self.arena
                        .node_at_mut(current_index)
                        .unwrap()
                        .push_child(char, next_index);
                    current_index = next_index;
                }
            };
        }
        let node = self.arena.node_at_mut(current_index).unwrap();
        node.set_value(value);
    }

    pub(crate) fn get_values_for_string(
        &self,
        word: &str,
        leeway: GetValuesOption,
    ) -> Option<Vec<(u8, U)>> {
        let mut curr = self.arena.root;

        for char in word.chars() {
            match self
                .arena
                .node_at(curr.to_owned())
                .and_then(|node| node.get_child(&char))
            {
                Some(new_node) => curr = *new_node,
                None => return None,
            }
        }

        match leeway {
            GetValuesOption::Exact => self.arena.node_at(curr).map(|node| {
                node.get_values()
                    .into_iter()
                    .map(|value| (0, value))
                    .collect()
            }),
            GetValuesOption::All => Some(self.walk_node_at_index(curr).collect()),
            GetValuesOption::Take(n) => Some(self.walk_node_at_index(curr).take(n).collect()),
        }
        .map(|mut values: Vec<(u8, U)>| {
            values.sort_by_key(|t| t.1.clone());
            values.dedup_by_key(|t| t.1.clone());
            values
        })
    }

    fn walk_node_at_index(&self, node_index: usize) -> NodeValuesWalk<U> {
        NodeValuesWalk::from(self.arena.clone(), node_index)
    }
}

pub(crate) enum GetValuesOption {
    Exact,
    All,
    Take(usize),
}

struct NodeValuesWalk<U>
where
    U: NodeValueTrait,
{
    arena: Arena<Node<U>>,
    values_on_deck: VecDeque<(u8, U)>,
    nodes_to_inspect: VecDeque<(u8, arena::ArenaIndex)>,
    current_depth: u8,
}

impl<U> NodeValuesWalk<U>
where
    U: NodeValueTrait,
{
    fn from(arena: Arena<Node<U>>, node_index: usize) -> Self {
        Self {
            arena,
            values_on_deck: VecDeque::new(),
            nodes_to_inspect: VecDeque::from([(0, node_index)]),
            current_depth: 0,
        }
    }
}

impl<U> Iterator for NodeValuesWalk<U>
where
    U: NodeValueTrait,
{
    type Item = (u8, U);

    fn next(&mut self) -> Option<Self::Item> {
        while self.values_on_deck.is_empty() && !self.nodes_to_inspect.is_empty() {
            let (next_depth, next_inspectable_index) = self.nodes_to_inspect.pop_front().unwrap();
            let node = self.arena.node_at(next_inspectable_index).unwrap();

            for value in node.get_values() {
                self.values_on_deck.push_back((next_depth, value))
            }

            for arena_index in node.get_all_children() {
                self.nodes_to_inspect
                    .push_back((next_depth + 1, arena_index))
            }
        }

        self.values_on_deck.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use crate::index_v4::tree::arena::ArenaIndex;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn insert_values_two_words() {
        let mut tree = Tree::default();
        tree.push_value_for_string("test", 1);
        tree.push_value_for_string("tesseract", 2);

        assert_eq!(
            tree.get_values_for_string("t", GetValuesOption::Exact),
            None
        );
        assert_eq!(
            tree.get_values_for_string("te", GetValuesOption::Exact),
            None
        );
        assert_eq!(
            tree.get_values_for_string("tess", GetValuesOption::Exact),
            None
        );

        assert_eq!(
            tree.get_values_for_string("tes", GetValuesOption::All)
                .map(|mut vec| {
                    vec.sort_by_key(|tuple| tuple.1);
                    vec
                }),
            Some(vec![(6, 2), (1, 1)]).map(|mut vec| {
                vec.sort_by_key(|tuple| tuple.1);
                vec
            })
        );

        assert_eq!(
            tree.get_values_for_string("test", GetValuesOption::Exact),
            Some(vec![(0, 1)])
        );
        assert_eq!(
            tree.get_values_for_string("tesseract", GetValuesOption::Exact),
            Some(vec![(0, 2)])
        );
    }

    #[test]
    fn returns_none_when_no_result() {
        let mut tree = Tree::default();
        tree.push_value_for_string("hyperbolic", 42);
        assert_eq!(
            tree.get_values_for_string("tower", GetValuesOption::Exact),
            None
        );
    }

    #[test]
    fn walks_remaining_values_correctly() {
        let mut tree = Tree::default();
        tree.push_value_for_string("liberty", 10);
        tree.push_value_for_string("liberate", 20);
        let walked_values = tree
            .walk_node_at_index(0)
            .collect::<Vec<(u8, ArenaIndex)>>();
        assert_eq!(vec![(7, 10), (8, 20)], walked_values)
    }
}
