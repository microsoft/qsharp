// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod display;
pub mod functors;
pub mod index_map;
pub mod language_features;
pub mod line_column;
pub mod span;
pub mod namespaces {
    use std::{cell::RefCell, collections::HashMap, fmt::Display, iter::Peekable, ops::Deref, rc::Rc};

    #[derive(Debug, Clone)]

    pub struct NamespaceTreeRoot {
        assigner: usize,
        tree: NamespaceTreeNode,
    }

    /// An ID that corresponds to a namespace in the global scope.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Default)]
    pub struct NamespaceId(usize);
    impl NamespaceId {
        /// Create a new namespace ID.
        pub fn new(value: usize) -> Self {
            Self(value)
        }
    }

    impl From<usize> for NamespaceId {
        fn from(value: usize) -> Self {
            Self::new(value)
        }
    }

    impl From<NamespaceId> for usize {
        fn from(value: NamespaceId) -> Self {
            value.0
        }
    }

    impl From<&NamespaceId> for usize {
        fn from(value: &NamespaceId) -> Self {
            value.0
        }
    }

    impl Deref for NamespaceId {
        type Target = usize;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Display for NamespaceId {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "Namespace {}", self.0)
        }
    }

    impl NamespaceTreeRoot {
        /// Create a new namespace tree root. The assigner is used to assign new namespace IDs.
        pub fn new(assigner: usize, tree: NamespaceTreeNode) -> Self {
            Self { assigner, tree }
        }
        /// Get the namespace tree field. This is the root of the namespace tree.
        pub fn tree(&self) -> &NamespaceTreeNode {
            &self.tree
        }

        /// Insert a namespace into the tree. If the namespace already exists, return its ID.
        pub fn insert_or_find_namespace(
            &mut self,
            ns: impl IntoIterator<Item = Rc<str>>,
        ) -> NamespaceId {
            self.tree
                .insert_or_find_namespace(ns.into_iter().peekable(), &mut self.assigner)
                .expect("namespace creation should not fail")
        }

        pub fn new_namespace_node(
            &mut self,
            children: HashMap<Rc<str>, NamespaceTreeNode>,
        ) -> NamespaceTreeNode {
            self.assigner += 1;
            NamespaceTreeNode {
                id: NamespaceId::new(self.assigner),
                children,
            }
        }

        pub fn find_namespace(&self, ns: impl Into<Vec<Rc<str>>>) -> Option<NamespaceId> {
            self.tree.find_namespace(ns)
        }

        pub fn find_id(&self, id: &NamespaceId) -> (Vec<Rc<str>>, Rc<&NamespaceTreeNode>) {
            return self.tree.find_id(*id, vec![]);
        }
        
        pub fn root_id(&self) -> NamespaceId {
            self.tree.id
        }
    }

    // write some unit tests for the above `find_id` method
    #[cfg(test)]
    mod tests {
        use expect_test::expect;

        use super::*;
        use std::collections::HashMap;

        #[test]
        fn test_tree_construction() {
            let mut root = NamespaceTreeRoot::default();
            for i in 0..10 {
                for j in 'a'..'d' {
                    root.insert_or_find_namespace(vec![Rc::from(format!("ns{}", i)), Rc::from(format!("ns{}", j))].into_iter());
                }
            }            
            expect![[r#"
                NamespaceTreeRoot {
                    assigner: 40,
                    tree: NamespaceTreeNode {
                        children: {
                            "ns6": NamespaceTreeNode {
                                children: {
                                    "nsb": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            27,
                                        ),
                                    },
                                    "nsc": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            28,
                                        ),
                                    },
                                    "nsa": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            26,
                                        ),
                                    },
                                },
                                id: NamespaceId(
                                    25,
                                ),
                            },
                            "ns3": NamespaceTreeNode {
                                children: {
                                    "nsa": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            14,
                                        ),
                                    },
                                    "nsc": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            16,
                                        ),
                                    },
                                    "nsb": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            15,
                                        ),
                                    },
                                },
                                id: NamespaceId(
                                    13,
                                ),
                            },
                            "ns7": NamespaceTreeNode {
                                children: {
                                    "nsc": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            32,
                                        ),
                                    },
                                    "nsb": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            31,
                                        ),
                                    },
                                    "nsa": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            30,
                                        ),
                                    },
                                },
                                id: NamespaceId(
                                    29,
                                ),
                            },
                            "ns8": NamespaceTreeNode {
                                children: {
                                    "nsb": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            35,
                                        ),
                                    },
                                    "nsc": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            36,
                                        ),
                                    },
                                    "nsa": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            34,
                                        ),
                                    },
                                },
                                id: NamespaceId(
                                    33,
                                ),
                            },
                            "ns0": NamespaceTreeNode {
                                children: {
                                    "nsb": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            3,
                                        ),
                                    },
                                    "nsa": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            2,
                                        ),
                                    },
                                    "nsc": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            4,
                                        ),
                                    },
                                },
                                id: NamespaceId(
                                    1,
                                ),
                            },
                            "ns4": NamespaceTreeNode {
                                children: {
                                    "nsc": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            20,
                                        ),
                                    },
                                    "nsa": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            18,
                                        ),
                                    },
                                    "nsb": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            19,
                                        ),
                                    },
                                },
                                id: NamespaceId(
                                    17,
                                ),
                            },
                            "ns2": NamespaceTreeNode {
                                children: {
                                    "nsa": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            10,
                                        ),
                                    },
                                    "nsb": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            11,
                                        ),
                                    },
                                    "nsc": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            12,
                                        ),
                                    },
                                },
                                id: NamespaceId(
                                    9,
                                ),
                            },
                            "ns5": NamespaceTreeNode {
                                children: {
                                    "nsa": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            22,
                                        ),
                                    },
                                    "nsb": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            23,
                                        ),
                                    },
                                    "nsc": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            24,
                                        ),
                                    },
                                },
                                id: NamespaceId(
                                    21,
                                ),
                            },
                            "ns1": NamespaceTreeNode {
                                children: {
                                    "nsc": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            8,
                                        ),
                                    },
                                    "nsa": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            6,
                                        ),
                                    },
                                    "nsb": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            7,
                                        ),
                                    },
                                },
                                id: NamespaceId(
                                    5,
                                ),
                            },
                            "ns9": NamespaceTreeNode {
                                children: {
                                    "nsa": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            38,
                                        ),
                                    },
                                    "nsb": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            39,
                                        ),
                                    },
                                    "nsc": NamespaceTreeNode {
                                        children: {},
                                        id: NamespaceId(
                                            40,
                                        ),
                                    },
                                },
                                id: NamespaceId(
                                    37,
                                ),
                            },
                        },
                        id: NamespaceId(
                            0,
                        ),
                    },
                }
            "#]]
            .assert_debug_eq(&root);
        }
        
        #[test]
        fn test_find_id() {
            let mut root = NamespaceTreeRoot::default();
            let mut id_buf = vec![];
            for i in 0..10 {
                for j in 'a'..'d' {
                    id_buf.push(root.insert_or_find_namespace(vec![Rc::from(format!("ns{}", i)), Rc::from(format!("ns{}", j))].into_iter()));
                }
            }
            let mut result_buf = vec![];
            for id in id_buf {
                result_buf.push(root.find_id(&id));
            }
            expect![[r#"
                [
                    (
                        [
                            "ns0",
                            "nsa",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                2,
                            ),
                        },
                    ),
                    (
                        [
                            "ns0",
                            "nsb",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                3,
                            ),
                        },
                    ),
                    (
                        [
                            "ns0",
                            "nsc",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                4,
                            ),
                        },
                    ),
                    (
                        [
                            "ns1",
                            "nsa",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                6,
                            ),
                        },
                    ),
                    (
                        [
                            "ns1",
                            "nsb",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                7,
                            ),
                        },
                    ),
                    (
                        [
                            "ns1",
                            "nsc",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                8,
                            ),
                        },
                    ),
                    (
                        [
                            "ns2",
                            "nsa",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                10,
                            ),
                        },
                    ),
                    (
                        [
                            "ns2",
                            "nsb",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                11,
                            ),
                        },
                    ),
                    (
                        [
                            "ns2",
                            "nsc",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                12,
                            ),
                        },
                    ),
                    (
                        [
                            "ns3",
                            "nsa",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                14,
                            ),
                        },
                    ),
                    (
                        [
                            "ns3",
                            "nsb",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                15,
                            ),
                        },
                    ),
                    (
                        [
                            "ns3",
                            "nsc",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                16,
                            ),
                        },
                    ),
                    (
                        [
                            "ns4",
                            "nsa",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                18,
                            ),
                        },
                    ),
                    (
                        [
                            "ns4",
                            "nsb",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                19,
                            ),
                        },
                    ),
                    (
                        [
                            "ns4",
                            "nsc",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                20,
                            ),
                        },
                    ),
                    (
                        [
                            "ns5",
                            "nsa",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                22,
                            ),
                        },
                    ),
                    (
                        [
                            "ns5",
                            "nsb",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                23,
                            ),
                        },
                    ),
                    (
                        [
                            "ns5",
                            "nsc",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                24,
                            ),
                        },
                    ),
                    (
                        [
                            "ns6",
                            "nsa",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                26,
                            ),
                        },
                    ),
                    (
                        [
                            "ns6",
                            "nsb",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                27,
                            ),
                        },
                    ),
                    (
                        [
                            "ns6",
                            "nsc",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                28,
                            ),
                        },
                    ),
                    (
                        [
                            "ns7",
                            "nsa",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                30,
                            ),
                        },
                    ),
                    (
                        [
                            "ns7",
                            "nsb",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                31,
                            ),
                        },
                    ),
                    (
                        [
                            "ns7",
                            "nsc",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                32,
                            ),
                        },
                    ),
                    (
                        [
                            "ns8",
                            "nsa",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                34,
                            ),
                        },
                    ),
                    (
                        [
                            "ns8",
                            "nsb",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                35,
                            ),
                        },
                    ),
                    (
                        [
                            "ns8",
                            "nsc",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                36,
                            ),
                        },
                    ),
                    (
                        [
                            "ns9",
                            "nsa",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                38,
                            ),
                        },
                    ),
                    (
                        [
                            "ns9",
                            "nsb",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                39,
                            ),
                        },
                    ),
                    (
                        [
                            "ns9",
                            "nsc",
                        ],
                        NamespaceTreeNode {
                            children: {},
                            id: NamespaceId(
                                40,
                            ),
                        },
                    ),
                ]
            "#]   ].assert_debug_eq(&result_buf)     
        }
        // test that after inserting lots of namespaces, all ids are unique and sequential
        #[test]
        fn test_insert_or_find_namespace() {
            let mut root = NamespaceTreeRoot::default();
            let mut ids: Vec<usize> = vec![];
            for i in 0..10 {
                for j in 'a'..'d' {
                    let id = root.insert_or_find_namespace(vec![Rc::from(format!("ns{}", i)), Rc::from(format!("ns{}", j))].into_iter());
                    ids.push(id.into());
                }
            }
            let mut ids_sorted = ids.clone();
            ids_sorted.sort();
            ids_sorted.dedup();
            // there should be no duplicate or out-of-order ids
            assert_eq!(ids_sorted, ids);
            expect![[r#"
                [
                    2,
                    3,
                    4,
                    6,
                    7,
                    8,
                    10,
                    11,
                    12,
                    14,
                    15,
                    16,
                    18,
                    19,
                    20,
                    22,
                    23,
                    24,
                    26,
                    27,
                    28,
                    30,
                    31,
                    32,
                    34,
                    35,
                    36,
                    38,
                    39,
                    40,
                ]
            "#]]
            .assert_debug_eq(&ids);
        }
    }
    impl Default for NamespaceTreeRoot {
        fn default() -> Self {
            Self {
                assigner: 0,
                tree: NamespaceTreeNode {
                    children: HashMap::new(),
                    id: NamespaceId::new(0),
                },
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct NamespaceTreeNode {
        pub children: HashMap<Rc<str>, NamespaceTreeNode>,
        pub id: NamespaceId,
    }
    impl NamespaceTreeNode {
        pub fn new(id: NamespaceId, children: HashMap<Rc<str>, NamespaceTreeNode>) -> Self {
            Self { children, id }
        }
        pub fn children(&self) -> &HashMap<Rc<str>, NamespaceTreeNode> {
            &self.children
        }
        fn get(&self, component: &Rc<str>) -> Option<&NamespaceTreeNode> {
            self.children.get(component)
        }
        pub fn id(&self) -> NamespaceId {
            self.id
        }
        fn contains(&self, ns: impl Into<Vec<Rc<str>>>) -> bool {
            self.find_namespace(ns).is_some()
        }
        fn find_namespace(&self, ns: impl Into<Vec<Rc<str>>>) -> Option<NamespaceId> {
            // look up a namespace in the tree and return the id
            // do a breadth-first search through NamespaceTree for the namespace
            // if it's not found, return None
            let mut buf = Rc::new(self);
            for component in ns.into().iter() {
                if let Some(next_ns) = buf.get(component) {
                    buf = Rc::new(next_ns);
                } else {
                    return None;
                }
            }
            return Some(buf.id);
        }

        /// If the namespace already exists, it will not be inserted.
        /// Returns the ID of the namespace.
        pub fn insert_or_find_namespace<I>(
            &mut self,
            mut iter:  Peekable<I>,
            assigner: &mut usize,
        ) -> Option<NamespaceId> where I: Iterator<Item = Rc<str>> {
            let next_item = match iter.next() {
                Some(item) => item,
                None => return None,
            };
            println!("Inserting namespace {}", next_item);

            let next_node = self.children.get_mut(&next_item);
            if let Some(mut next_node) = next_node {
                return next_node.insert_or_find_namespace(iter, assigner);
            } else {
                println!("creating new node");
                *assigner += 1;
                let mut new_node =
                    NamespaceTreeNode::new(NamespaceId::new(*assigner), HashMap::new());
                if iter.peek().is_none() {
                    let new_node_id = new_node.id;
                    self.children.insert(next_item, new_node);
                    return Some(new_node_id);
                } else {
                    let id = new_node.insert_or_find_namespace(iter, assigner);
                    self.children.insert(next_item, new_node);
                    return id;
                }
            }
        }
        
        fn find_id(&self, id: NamespaceId, names_buf: Vec<Rc<str>>) -> (Vec<Rc<str>>, Rc<&NamespaceTreeNode>) {
            if self.id == id {
                return (names_buf, Rc::new(self));
            } else {
                for (name, node) in self.children.iter() {
                    let mut new_buf = names_buf.clone();
                    new_buf.push(name.clone());
                    let (names, node) = node.find_id(id, new_buf);
                    if names.len() > 0 {
                        return (names, node);
                    }
                }
                return (vec![], Rc::new(self));
            }
        }
    }
}
