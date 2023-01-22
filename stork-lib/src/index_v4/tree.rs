use itertools::Itertools;
use minicbor::Decode;

#[cfg(feature = "build")]
use minicbor::Encode;

use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
};

#[cfg(test)]
use std::cmp;

pub(crate) type ArenaId = String;
pub(crate) type NodeIndex = usize;

#[cfg_attr(test, derive(Debug, PartialEq))]
struct NodePointer {
    arena_id: ArenaId,
    node_index: NodeIndex,
}

#[derive(Decode, Clone)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct Tree<U> {
    #[n(0)]
    arenas: HashMap<ArenaId, Arena<U>>,

    #[n(1)]
    root_arena: ArenaId,
}

#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(test, derive(Debug))]
#[derive(Clone)]
pub(crate) enum TreeRetrievalValue<U> {
    Value { value: U, characters_remaining: u8 },
    UnloadedArena(ArenaId),
}

#[cfg(test)]
impl<U> PartialOrd<TreeRetrievalValue<U>> for TreeRetrievalValue<U>
where
    U: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (
                TreeRetrievalValue::Value {
                    value,
                    characters_remaining,
                },
                TreeRetrievalValue::Value {
                    value: other_value,
                    characters_remaining: other_characters_remaining,
                },
            ) => match value.partial_cmp(other_value) {
                Some(cmp::Ordering::Equal) => {
                    characters_remaining.partial_cmp(other_characters_remaining)
                }
                _ => value.partial_cmp(other_value),
            },
            _ => None,
        }
    }
}

// * No results available, need chunk x
// * Some results available, need chunks [x, y, z] for more
// * Results available, need chunks [x, y, z] for excerpts

// #[cfg_attr(test, derive(Debug, PartialEq))]
// pub(crate) struct TreeRetrievalValue {
//     value: U,
//     characters_remaining: u8,
// }

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum TreeRetrievalError {
    ArenaNotLoaded(String),
    NoValueAtString,
    UnexpectedNodeIndexOutOfBounds,
}

impl<U> Tree<U>
where
    U: Clone,
{
    pub(crate) fn retrieve_values_for_string(
        &self,
        string: &str,
        exact: bool,
    ) -> Result<Vec<TreeRetrievalValue<U>>, TreeRetrievalError> {
        let (node_pointer, node) = self.walk_to_node_at_string(string)?;
        if exact {
            self.get_node_at_pointer(&node_pointer).map(|node| {
                node.values
                    .iter()
                    .map(|v| TreeRetrievalValue::Value {
                        value: v.clone(),
                        characters_remaining: 0,
                    })
                    .collect_vec()
            })
        } else {
            Ok(self.retrieve_values_from_children(node_pointer).collect())
        }
    }

    pub(crate) fn load_arenas(&mut self, other: &mut Vec<Arena<U>>) {
        for arena in other.drain(..) {
            self.arenas.insert(arena.id.clone(), arena);
        }
    }
}

#[cfg(feature = "build")]
impl<U> Tree<U>
where
    U: Clone,
{
    pub(crate) fn new() -> Self {
        Tree {
            arenas: HashMap::from([(
                "root".to_string(),
                Arena {
                    id: "root".to_string(),
                    nodes: vec![],
                },
            )]),
            root_arena: "root".to_string(),
        }
    }

    pub(crate) fn insert_value_for_string(&mut self, value: U, string: &str) {
        let mut node_pointer = NodePointer {
            arena_id: self.root_arena.clone(),
            node_index: 0,
        };

        if self.get_node_at_pointer(&node_pointer).is_err() {
            let arena = self.arenas.get_mut(&node_pointer.arena_id).unwrap();
            arena.nodes.push(Node::new());
            let node_index = arena.nodes.len() - 1;
            node_pointer.node_index = node_index;
        }

        for char in string.chars() {
            match self.arenas.get_mut(&node_pointer.arena_id) {
                Some(arena) => match arena.nodes.get(node_pointer.node_index) {
                    Some(node) => match node.children.get(&char) {
                        None => {
                            arena.nodes.push(Node::new());
                            let node_index = arena.nodes.len() - 1;
                            arena
                                .nodes
                                .get_mut(node_pointer.node_index)
                                .unwrap()
                                .children
                                .insert(char, NodeChildValue::Local(node_index));
                            node_pointer.node_index = node_index;
                        }
                        Some(node_child_value) => match node_child_value {
                            NodeChildValue::Remote(arena_id) => {
                                node_pointer.arena_id = arena_id.clone()
                            }
                            NodeChildValue::Local(node_index) => {
                                node_pointer.node_index = *node_index;
                            }
                        },
                    },
                    None => {
                        arena.nodes.push(Node::new());
                        let node_index = arena.nodes.len() - 1;
                        node_pointer.node_index = node_index;
                    }
                },
                None => panic!(),
            }
        }

        self.arenas.get_mut(&node_pointer.arena_id).map(|arena| {
            arena.nodes.get_mut(node_pointer.node_index).map(|node| {
                node.values.push(value);
            })
        });
    }

    pub(crate) fn segment_arenas(&mut self) {}
}

enum GetNodeMutRetval<'a, U> {
    Node(&'a mut Node<U>, &'a mut Arena<U>),
    Arena(&'a mut Arena<U>),
    NotFound,
}

// Private API
impl<U> Tree<U>
where
    U: Clone,
{
    fn get_node_at_pointer(
        &self,
        node_pointer: &NodePointer,
    ) -> Result<&Node<U>, TreeRetrievalError> {
        self.arenas
            .get(&node_pointer.arena_id)
            .ok_or_else(|| TreeRetrievalError::ArenaNotLoaded(node_pointer.arena_id.clone()))?
            .nodes
            .get(node_pointer.node_index)
            .ok_or(TreeRetrievalError::UnexpectedNodeIndexOutOfBounds)
    }

    // fn get_node_at_pointer_mut<'s>(
    //     &'s mut self,
    //     node_pointer: &NodePointer,
    // ) -> GetNodeMutRetval<'s, U> {
    //     match self.arenas.get_mut(&node_pointer.arena_id) {
    //         Some(arena) => {
    //             let nodes = &mut arena.nodes;
    //             match nodes.get_mut(node_pointer.node_index) {
    //                 Some(node) => GetNodeMutRetval::Node(node, arena),
    //                 None => GetNodeMutRetval::Arena(arena),
    //             }
    //         }
    //         None => GetNodeMutRetval::NotFound,
    //     }
    // }

    fn walk_to_node_at_string(
        &self,
        string: &str,
    ) -> Result<(NodePointer, &Node<U>), TreeRetrievalError> {
        let mut node_pointer = NodePointer {
            arena_id: self.root_arena.clone(),
            node_index: 0,
        };

        for char in string.chars() {
            match self.get_node_at_pointer(&node_pointer) {
                Ok(node) => match node.children.get(&char) {
                    Some(NodeChildValue::Remote(arena_id)) => {
                        node_pointer.arena_id = arena_id.to_owned();
                        node_pointer.node_index = 0;
                    }
                    Some(NodeChildValue::Local(node_index)) => {
                        node_pointer.node_index = *node_index;
                    }
                    None => return Err(TreeRetrievalError::NoValueAtString),
                },
                Err(e) => return Err(e),
            }
        }

        self.get_node_at_pointer(&node_pointer)
            .map(|node| (node_pointer, node))
    }

    fn retrieve_values_from_children(&self, node_pointer: NodePointer) -> NodeWalkIterator<U> {
        NodeWalkIterator::from(Cow::Borrowed(self), node_pointer)
    }
}

#[derive(Decode, Clone)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
#[cfg_attr(test, derive(PartialEq))]
pub struct Arena<U> {
    #[n(0)]
    id: ArenaId,

    #[n(1)]
    nodes: Vec<Node<U>>,
}

#[derive(Clone, Decode)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
enum NodeChildValue {
    #[n(0)]
    Remote(#[n(0)] ArenaId),

    #[n(1)]
    Local(#[n(0)] NodeIndex),
}

#[derive(Decode, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub struct Node<U> {
    #[n(0)]
    values: Vec<U>,

    #[n(1)]
    children: HashMap<char, NodeChildValue>,
    // #[n(2)]
    // arena_ids_to_preload: Vec<ArenaId>, // TODO: Reinstate this field and use to make hints
}

#[cfg(feature = "build")]
impl<U> Node<U> {
    pub fn new() -> Self {
        Self {
            values: vec![],
            children: HashMap::new(),
        }
    }
}

struct NodeWalkIterator<'a, U>
where
    U: Clone,
{
    tree: Cow<'a, Tree<U>>,
    values_to_vend: VecDeque<TreeRetrievalValue<U>>,
    nodes_to_inspect: VecDeque<(u8, NodePointer)>,
    current_depth: u8,
}

impl<'a, U> NodeWalkIterator<'a, U>
where
    U: Clone,
{
    fn from(tree: Cow<'a, Tree<U>>, node_pointer: NodePointer) -> Self {
        Self {
            tree,
            values_to_vend: VecDeque::new(),
            nodes_to_inspect: VecDeque::from([(0, node_pointer)]),
            current_depth: 0,
        }
    }
}

impl<U> Iterator for NodeWalkIterator<'_, U>
where
    U: Clone,
{
    type Item = TreeRetrievalValue<U>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.values_to_vend.is_empty() && !self.nodes_to_inspect.is_empty() {
            let (next_depth, next_pointer_to_inspect) = self.nodes_to_inspect.pop_front().unwrap();
            let node_result = self.tree.get_node_at_pointer(&next_pointer_to_inspect);

            match node_result {
                Err(TreeRetrievalError::ArenaNotLoaded(arena)) => self
                    .values_to_vend
                    .push_back(TreeRetrievalValue::UnloadedArena(arena)),

                Ok(node) => {
                    for value in &node.values {
                        self.values_to_vend.push_back(TreeRetrievalValue::Value {
                            value: value.clone(),
                            characters_remaining: next_depth,
                        })
                    }

                    for child in node.children.values() {
                        match child {
                            NodeChildValue::Remote(arena_id) => self.nodes_to_inspect.push_back((
                                next_depth + 1,
                                NodePointer {
                                    arena_id: arena_id.clone(),
                                    node_index: 0,
                                },
                            )),
                            NodeChildValue::Local(node_index) => self.nodes_to_inspect.push_back((
                                next_depth + 1,
                                NodePointer {
                                    arena_id: next_pointer_to_inspect.arena_id.clone(),
                                    node_index: *node_index,
                                },
                            )),
                        }
                    }
                }
                _ => panic!("This should never happen"),
            };
        }

        self.values_to_vend.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    ///                                  ┌───────────────────── ARENA Y ──┐   ┌──────── ARENA X ─┐
    ///                  ┌────────┐      │                  ┌─────────┐   │   │ ┌─────────┐      │
    ///              ┌──▶│  5, 6  │      │              ┌──▶│  9, 10  │───┼a──┼▶│   ???   │      │
    /// ┌────────┐  b│   └────────┘      │ ┌────────┐  a│   └─────────┘   │   │ └─────────┘      │
    /// │  1, 2  │───┤                ┌──┼▶│  7, 8  │───┤                 │   │                  │
    /// └────────┘  a│   ┌────────┐  a│  │ └────────┘  b│   ┌─────────┐   │   └──────────────────┘
    ///              └──▶│  3, 4  │───┤  │              └──▶│ 13, 14  │   │
    ///                  └────────┘   │  │                  └─────────┘   │
    ///                              b│  └────────────────────────────────┘
    ///                               │
    ///                               │    ┌────────┐
    ///                               └───▶│ 11, 12 │
    ///                                    └────────┘
    ///
    fn make_tree() -> Tree<usize> {
        Tree {
            arenas: HashMap::from([
                (
                    "z".to_string(),
                    Arena {
                        id: "z".to_string(),
                        nodes: vec![
                            Node {
                                values: vec![1, 2],
                                children: HashMap::from([
                                    ('a', NodeChildValue::Local(1)),
                                    ('b', NodeChildValue::Local(2)),
                                ]),
                            },
                            Node {
                                values: vec![3, 4],
                                children: HashMap::from([
                                    ('a', NodeChildValue::Remote('y'.to_string())),
                                    ('b', NodeChildValue::Local(3)),
                                ]),
                            },
                            Node {
                                values: vec![5, 6],
                                children: HashMap::new(),
                            },
                            Node {
                                values: vec![11, 12],
                                children: HashMap::new(),
                            },
                        ],
                    },
                ),
                (
                    "y".to_string(),
                    Arena {
                        id: "y".to_string(),
                        nodes: vec![
                            Node {
                                values: vec![7, 8],
                                children: HashMap::from([
                                    ('a', NodeChildValue::Local(1)),
                                    ('b', NodeChildValue::Local(2)),
                                ]),
                            },
                            Node {
                                values: vec![9, 10],
                                children: HashMap::from([(
                                    'a',
                                    NodeChildValue::Remote("x".to_string()),
                                )]),
                            },
                            Node {
                                values: vec![13, 14],
                                children: HashMap::new(),
                            },
                        ],
                    },
                ),
            ]),
            root_arena: "z".to_string(),
        }
    }

    #[test]
    fn it_gets_the_root_node_given_a_pointer() {
        let tree = make_tree();
        assert_eq!(
            tree.get_node_at_pointer(&NodePointer {
                arena_id: 'z'.to_string(),
                node_index: 0
            })
            .unwrap()
            .values,
            vec![1, 2],
        );
    }

    #[test]
    fn it_gets_an_arbitrary_node_given_a_pointer() {
        let tree = make_tree();

        assert_eq!(
            tree.get_node_at_pointer(&NodePointer {
                arena_id: 'y'.to_string(),
                node_index: 1
            })
            .unwrap()
            .values,
            vec![9, 10]
        );
    }

    #[test]
    fn it_walks_to_a_tree_through_local_path() {
        let tree = make_tree();

        assert_eq!(
            tree.walk_to_node_at_string("ab").unwrap().1.values,
            vec![11, 12],
        );
    }

    #[test]
    fn it_walks_to_a_tree_through_remote_path() {
        let tree = make_tree();

        assert_eq!(
            tree.walk_to_node_at_string("aaa").unwrap().1.values,
            vec![9, 10],
        );
    }

    #[test]
    fn it_errors_if_the_path_doesnt_exist() {
        let tree = make_tree();

        assert_eq!(
            tree.walk_to_node_at_string("abc"),
            Err(TreeRetrievalError::NoValueAtString),
        );
    }

    #[test]
    fn it_errors_if_the_arena_isnt_loaded() {
        let tree = make_tree();

        assert_eq!(
            tree.walk_to_node_at_string("aaaa"),
            Err(TreeRetrievalError::ArenaNotLoaded("x".to_string())),
        );
    }

    #[test]
    #[ignore = "reason"]
    fn it_collects_values_from_children() {
        let tree = make_tree();

        assert_eq!(
            tree.retrieve_values_from_children(NodePointer {
                arena_id: "z".to_string(),
                node_index: 1
            })
            .collect::<Vec<TreeRetrievalValue<usize>>>(),
            vec![
                TreeRetrievalValue::Value {
                    value: 3,
                    characters_remaining: 0
                },
                TreeRetrievalValue::Value {
                    value: 4,
                    characters_remaining: 0
                },
                TreeRetrievalValue::Value {
                    value: 7,
                    characters_remaining: 1
                },
                TreeRetrievalValue::Value {
                    value: 8,
                    characters_remaining: 1
                },
                TreeRetrievalValue::Value {
                    value: 11,
                    characters_remaining: 1
                },
                TreeRetrievalValue::Value {
                    value: 12,
                    characters_remaining: 1
                },
                TreeRetrievalValue::Value {
                    value: 9,
                    characters_remaining: 2
                },
                TreeRetrievalValue::Value {
                    value: 10,
                    characters_remaining: 2
                },
                TreeRetrievalValue::Value {
                    value: 13,
                    characters_remaining: 2
                },
                TreeRetrievalValue::Value {
                    value: 14,
                    characters_remaining: 2
                },
                TreeRetrievalValue::UnloadedArena("x".to_string()),
            ]
        );
    }

    fn make_simple_tree() -> Tree<usize> {
        Tree {
            root_arena: "root".to_string(),
            arenas: HashMap::from([(
                "root".to_string(),
                Arena {
                    id: "root".to_string(),
                    nodes: vec![
                        Node {
                            values: vec![1, 2],
                            children: HashMap::from([
                                ('b', NodeChildValue::Local(2)),
                                ('a', NodeChildValue::Local(1)),
                            ]),
                        },
                        Node {
                            values: vec![3, 4],
                            children: HashMap::from([('a', NodeChildValue::Local(3))]),
                        },
                        Node {
                            values: vec![5, 6],
                            children: HashMap::new(),
                        },
                        Node {
                            values: vec![7, 8],
                            children: HashMap::new(),
                        },
                    ],
                },
            )]),
        }
    }

    #[test]
    fn it_pushes_values_into_children() {
        let mut tree: Tree<usize> = Tree::new();
        tree.insert_value_for_string(1, "");
        tree.insert_value_for_string(2, "");
        tree.insert_value_for_string(3, "a");
        tree.insert_value_for_string(4, "a");
        tree.insert_value_for_string(5, "b");
        tree.insert_value_for_string(6, "b");
        tree.insert_value_for_string(7, "aa");
        tree.insert_value_for_string(8, "aa");

        assert_eq!(tree, make_simple_tree())
    }
}
