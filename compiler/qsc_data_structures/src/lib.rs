// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod display;
pub mod functors;
pub mod index_map;
pub mod language_features;
pub mod line_column;
pub mod span;
pub mod namespaces {
    use std::{cell::RefCell, collections::HashMap, fmt::Display, ops::Deref, rc::Rc};

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
        /// Upserts a namespace into the tree. If the namespace already exists, it will not be inserted. 
        /// Returns the ID of the namespace.
        pub fn upsert_namespace(&mut self, name: impl Into<Vec<Rc<str>>>) -> NamespaceId {
            self.assigner += 1;
            let id = self.assigner;
            let node = self.new_namespace_node(Default::default());
            let components_iter = name.into();
            let components_iter = components_iter.iter();
            // construct the initial rover for the breadth-first insertion
            // (this is like a BFS but we create a new node if one doesn't exist)
            let self_cell = RefCell::new(self);
            let mut rover_node = &mut self_cell.borrow_mut().tree;
            // create the rest of the nodes
            for component in components_iter {
                rover_node = rover_node
                    .children
                    .entry(Rc::clone(component))
                    .or_insert_with(|| {
                        self_cell
                            .borrow_mut()
                            .new_namespace_node(Default::default())
                    });
            }
            rover_node.id
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
            Self {
                children,
                id,
            }
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
    }
}
