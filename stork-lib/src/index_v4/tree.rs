use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
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

    pub fn add_node(&mut self, node: T) -> ArenaIndex {
        let index = self.values.len();
        self.values.push(Some(node));
        index
    }

    pub fn remove_node_at(&mut self, index: ArenaIndex) -> Option<T> {
        if let Some(node) = self.values.get_mut(index) {
            node.take()
        } else {
            None
        }
    }

    pub fn node_at(&self, index: ArenaIndex) -> Option<&T> {
        return if let Some(node) = self.values.get(index) {
            node.as_ref()
        } else {
            None
        };
    }

    pub fn node_at_mut(&mut self, index: ArenaIndex) -> Option<&mut T> {
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
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct CharEdgeTree<U>
where
    U: Debug + Clone,
{
    arena: Arena<CharEdgeNode<Vec<U>>>,
}

impl<U> Default for CharEdgeTree<U>
where
    U: Debug + Clone,
{
    fn default() -> Self {
        CharEdgeTree {
            arena: Arena {
                values: vec![Some(CharEdgeTree::build_node())],
                root: Some(0),
            },
        }
    }
}

impl<U> CharEdgeTree<U>
where
    U: Debug + Clone,
{
    fn build_node() -> CharEdgeNode<Vec<U>> {
        CharEdgeNode {
            value: vec![],
            children: BTreeMap::default(),
        }
    }

    pub(crate) fn push_value_for_word(&mut self, word: &str, value: U) {
        let mut current_index = self.arena.root.unwrap();
        for char in word.chars() {
            let next_index = self
                .arena
                .node_at(current_index)
                .unwrap()
                .children
                .get(&char);
            if let Some(next_index) = next_index {
                current_index = next_index.to_owned();
            } else {
                let mut new_node: CharEdgeNode<Vec<U>> = CharEdgeTree::build_node();
                new_node.value.push(value.clone());
                let next_index = self.arena.add_node(CharEdgeTree::build_node());
                self.arena
                    .node_at_mut(current_index)
                    .unwrap()
                    .children
                    .insert(char, next_index);
                current_index = next_index;
            }
        }

        let current = self.arena.node_at_mut(current_index).unwrap();
        current.value.push(value);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn create_new_tree() {
//         let tree = Tree::new();
//         tree.push_value(1);

//         assert_eq!(tree.values, vec![1]);
//     }

//     #[test]
//     fn insert_value_and_child() {
//         let mut tree = Tree::new();
//         tree.push_value(1);
//         tree.push_value(2);
//         let child = tree.get_or_push_child('a');
//         child.push_value(3);

//         assert_eq!(
//             &tree,
//             &Tree {
//                 values: vec![1, 2],
//                 children: BTreeMap::from([(
//                     'a',
//                     Box::new(Tree {
//                         values: vec![3],
//                         children: BTreeMap::new()
//                     })
//                 )])
//             }
//         );
//     }

//     #[test]
//     fn insert_multiple_values_for_same_child() {
//         let mut tree = Tree::new();
//         tree.push_value(1);

//         let child = tree.get_or_push_child('a');
//         child.push_value(2);

//         let child = tree.get_or_push_child('a');
//         child.push_value(3);

//         assert_eq!(
//             &tree,
//             &Tree {
//                 values: vec![1],
//                 children: BTreeMap::from([(
//                     'a',
//                     Box::new(Tree {
//                         values: vec![2, 3],
//                         children: BTreeMap::new()
//                     })
//                 )])
//             }
//         );
//     }

//     // #[test]
//     // fn insert_right() {
//     //     let tree = BinaryTree::new(1).right(BinaryTree::new(2));

//     //     if let Some(node) = tree.right {
//     //         assert_eq!(node.value, 2);
//     //     }

//     //     assert_eq!(tree.left, None);
//     // }
// }
