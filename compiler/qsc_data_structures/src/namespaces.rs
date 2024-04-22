#[cfg(test)]
mod tests;

use rustc_hash::FxHashMap;
use std::{fmt::Display, iter::Peekable, ops::Deref, rc::Rc};

pub const PRELUDE: [[&str; 3]; 4] = [
    ["Microsoft", "Quantum", "Canon"],
    ["Microsoft", "Quantum", "Core"],
    ["Microsoft", "Quantum", "Intrinsic"],
    ["Microsoft", "Quantum", "Measurement"],
];

/// An ID that corresponds to a namespace in the global scope.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Default)]
pub struct NamespaceId(usize);
impl NamespaceId {
    /// Create a new namespace ID.
    #[must_use]
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

#[derive(Debug, Clone)]
pub struct NamespaceTreeRoot {
    assigner: usize,
    tree: NamespaceTreeNode,
}

impl NamespaceTreeRoot {
    /// Create a new namespace tree root. The assigner is used to assign new namespace IDs.
    #[must_use]
    pub fn new_from_parts(assigner: usize, tree: NamespaceTreeNode) -> Self {
        Self { assigner, tree }
    }

    /// Get the namespace tree field. This is the root of the namespace tree.
    #[must_use]
    // #[inline(never)]
    pub fn tree(&self) -> &NamespaceTreeNode {
        &self.tree
    }

    /// Insert a namespace into the tree. If the namespace already exists, return its ID.
    // #[inline(never)]

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
        children: FxHashMap<Rc<str>, NamespaceTreeNode>,
    ) -> NamespaceTreeNode {
        self.assigner += 1;
        NamespaceTreeNode {
            id: NamespaceId::new(self.assigner),
            children,
        }
    }
    // #[inline(never)]
    pub fn find_namespace(&self, ns: impl Into<Vec<Rc<str>>>) -> Option<NamespaceId> {
        self.tree.find_namespace(ns)
    }
    // #[inline(never)]

    #[must_use]
    pub fn find_id(&self, id: &NamespaceId) -> (Vec<Rc<str>>, &NamespaceTreeNode) {
        return self.tree.find_id(*id, &[]);
    }
    // #[inline(never)]

    #[must_use]
    pub fn root_id(&self) -> NamespaceId {
        self.tree.id
    }
}

impl Default for NamespaceTreeRoot {
    fn default() -> Self {
        let mut tree = Self {
            assigner: 0,
            tree: NamespaceTreeNode {
                children: FxHashMap::default(),
                id: NamespaceId::new(0),
            },
        };
        // insert the prelude namespaces using the `NamespaceTreeRoot` API
        for ns in &PRELUDE {
            let iter = ns.iter().map(|s| Rc::from(*s)).peekable();
            tree.insert_or_find_namespace(iter);
        }
        tree
    }
}

#[derive(Debug, Clone)]
pub struct NamespaceTreeNode {
    pub children: FxHashMap<Rc<str>, NamespaceTreeNode>,
    pub id: NamespaceId,
}
impl NamespaceTreeNode {
    #[must_use]
    pub fn new(id: NamespaceId, children: FxHashMap<Rc<str>, NamespaceTreeNode>) -> Self {
        Self { children, id }
    }

    #[must_use]
    pub fn children(&self) -> &FxHashMap<Rc<str>, NamespaceTreeNode> {
        &self.children
    }

    fn get(&self, component: &Rc<str>) -> Option<&NamespaceTreeNode> {
        self.children.get(component)
    }

    #[must_use]
    pub fn id(&self) -> NamespaceId {
        self.id
    }

    #[must_use]
    pub fn contains(&self, ns: impl Into<Vec<Rc<str>>>) -> bool {
        self.find_namespace(ns).is_some()
    }

    pub fn find_namespace(&self, ns: impl Into<Vec<Rc<str>>>) -> Option<NamespaceId> {
        // look up a namespace in the tree and return the id
        // do a breadth-first search through NamespaceTree for the namespace
        // if it's not found, return None
        let mut buf = Rc::new(self);
        for component in &ns.into() {
            if let Some(next_ns) = buf.get(component) {
                buf = Rc::new(next_ns);
            } else {
                return None;
            }
        }
        Some(buf.id)
    }

    /// If the namespace already exists, it will not be inserted.
    /// Returns the ID of the namespace.
    pub fn insert_or_find_namespace<I>(
        &mut self,
        mut iter: Peekable<I>,
        assigner: &mut usize,
    ) -> Option<NamespaceId>
    where
        I: Iterator<Item = Rc<str>>,
    {
        let next_item = iter.next()?;
        let next_node = self.children.get_mut(&next_item);
        match (next_node, iter.peek()) {
            (Some(next_node), Some(_)) => {
                return next_node.insert_or_find_namespace(iter, assigner);
            }
            (Some(next_node), None) => {
                return Some(next_node.id);
            }
            _ => {}
        }
        *assigner += 1;
        let mut new_node =
            NamespaceTreeNode::new(NamespaceId::new(*assigner), FxHashMap::default());
        if iter.peek().is_none() {
            let new_node_id = new_node.id;
            self.children.insert(next_item, new_node);
            Some(new_node_id)
        } else {
            let id = new_node.insert_or_find_namespace(iter, assigner);
            self.children.insert(next_item, new_node);
            id
        }
    }

    fn find_id(
        &self,
        id: NamespaceId,
        names_buf: &[Rc<str>],
    ) -> (Vec<Rc<str>>, &NamespaceTreeNode) {
        if self.id == id {
            return (names_buf.to_vec(), &self);
        }
        for (name, node) in &self.children {
            let (names, node) = node.find_id(id, names_buf);
            if !names.is_empty() {
                let mut names = names_buf.to_vec();
                names.push(name.clone());
                return (names, node);
            }
        }
        (vec![], &self)
    }
}
