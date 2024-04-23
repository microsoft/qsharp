#[cfg(test)]
mod tests;

use rustc_hash::FxHashMap;
use std::{cell::RefCell, fmt::Display, iter::Peekable, ops::Deref, rc::Rc};

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

/// A reference counted cell that supports interior mutability for namespace tree nodes.
/// Interior mutability is required to update the tree when inserting new data structures.
type NamespaceTreeCell = Rc<RefCell<NamespaceTreeNode>>;

/// An entry in the memoization table for namespace ID lookups.
type MemoEntry = (Vec<Rc<str>>, NamespaceTreeCell);

#[derive(Debug, Clone)]
pub struct NamespaceTreeRoot {
    assigner: usize,
    tree: NamespaceTreeCell,
    memo: RefCell<FxHashMap<NamespaceId, MemoEntry>>,
}

impl NamespaceTreeRoot {
    /// Create a new namespace tree root. The assigner is used to assign new namespace IDs.
    #[must_use]
    pub fn new_from_parts(assigner: usize, tree: NamespaceTreeNode) -> Self {
        Self {
            assigner,
            tree: Rc::new(RefCell::new(tree)),
            memo: RefCell::new(FxHashMap::default()),
        }
    }

    /// Get the namespace tree field. This is the root of the namespace tree.
    #[must_use]
    pub fn tree(&self) -> NamespaceTreeCell {
        self.tree.clone()
    }

    /// Insert a namespace into the tree. If the namespace already exists, return its ID.
    pub fn insert_or_find_namespace(
        &mut self,
        ns: impl IntoIterator<Item = Rc<str>>,
    ) -> NamespaceId {
        self.tree
            .borrow_mut()
            .insert_or_find_namespace(ns.into_iter().peekable(), &mut self.assigner)
            .expect("namespace creation should not fail")
    }

    pub fn new_namespace_node(
        &mut self,
        children: FxHashMap<Rc<str>, NamespaceTreeCell>,
    ) -> NamespaceTreeNode {
        self.assigner += 1;
        NamespaceTreeNode {
            id: NamespaceId::new(self.assigner),
            children,
        }
    }

    pub fn find_namespace(&self, ns: impl Into<Vec<Rc<str>>>) -> Option<NamespaceId> {
        self.tree.borrow().find_namespace(ns)
    }

    #[must_use]
    pub fn find_id(&self, id: &NamespaceId) -> (Vec<Rc<str>>, NamespaceTreeCell) {
        if let Some(res) = self.memo.borrow().get(id) {
            return res.clone();
        }
        let Some((names, node)) = self.tree.borrow().find_id(*id, &[]) else {
            return (vec![], self.tree.clone());
        };

        self.memo
            .borrow_mut()
            .insert(*id, (names.clone(), node.clone()));
        (names, node.clone())
    }

    #[must_use]
    pub fn root_id(&self) -> NamespaceId {
        self.tree.borrow().id
    }
}

impl Default for NamespaceTreeRoot {
    fn default() -> Self {
        let mut tree = Self {
            assigner: 0,
            tree: Rc::new(RefCell::new(NamespaceTreeNode {
                children: FxHashMap::default(),
                id: NamespaceId::new(0),
            })),
            memo: RefCell::new(FxHashMap::default()),
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
    pub children: FxHashMap<Rc<str>, NamespaceTreeCell>,
    pub id: NamespaceId,
}
impl NamespaceTreeNode {
    #[must_use]
    pub fn new(id: NamespaceId, children: FxHashMap<Rc<str>, NamespaceTreeCell>) -> Self {
        Self { children, id }
    }

    #[must_use]
    pub fn children(&self) -> &FxHashMap<Rc<str>, NamespaceTreeCell> {
        &self.children
    }

    fn get(&self, component: &Rc<str>) -> Option<NamespaceTreeCell> {
        self.children.get(component).cloned()
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

        let mut buf: Option<NamespaceTreeCell> = None;
        for component in &ns.into() {
            if let Some(next_ns) = match buf {
                None => self.get(component),
                Some(buf) => buf.borrow().get(component),
            } {
                buf = Some(next_ns);
            } else {
                return None;
            }
        }
        Some(buf.map_or_else(|| self.id, |x| x.borrow().id))
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
                return next_node
                    .borrow_mut()
                    .insert_or_find_namespace(iter, assigner);
            }
            (Some(next_node), None) => {
                return Some(next_node.borrow().id);
            }
            _ => {}
        }
        *assigner += 1;
        let mut new_node =
            NamespaceTreeNode::new(NamespaceId::new(*assigner), FxHashMap::default());
        if iter.peek().is_none() {
            let new_node_id = new_node.id;
            self.children
                .insert(next_item, Rc::new(RefCell::new(new_node)));
            Some(new_node_id)
        } else {
            let id = new_node.insert_or_find_namespace(iter, assigner);
            self.children
                .insert(next_item, Rc::new(RefCell::new(new_node)));
            id
        }
    }

    /// given a namespace id, find its name and node
    fn find_id(
        &self,
        id: NamespaceId,
        names_buf: &[Rc<str>],
    ) -> Option<(Vec<Rc<str>>, NamespaceTreeCell)> {
        // first, check if any children are the one we are looking for
        for (name, node) in &self.children {
            if node.borrow().id == id {
                let mut names = names_buf.to_vec();
                names.push(name.clone());
                return Some((names, node.clone()));
            }
        }

        // if it wasn't found, recurse into children
        for (name, node) in &self.children {
            let mut names = names_buf.to_vec();
            names.push(name.clone());
            let Some((names, node)) = node.borrow().find_id(id, &names) else {
                continue;
            };
            if !names.is_empty() {
                return Some((names, node));
            }
        }

        None
    }
}
