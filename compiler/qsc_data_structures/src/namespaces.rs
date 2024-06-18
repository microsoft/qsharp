// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use rustc_hash::{FxHashMap, FxHashSet};
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

/// The root of the data structure that represents the namespaces in a program.
/// The tree is a trie (prefix tree) where each node is a namespace and the children are the sub-namespaces.
/// For example, the namespace `Microsoft.Quantum.Canon` would be represented as a traversal from the root node:
/// ```
/// root
/// └ Microsoft
///   └ Quantum
///     └ Canon
/// ```
/// This data structure is optimized for looking up namespace IDs by a given name. Looking up a namespace name by ID is
/// less efficient, as it performs a breadth-first search. Because of this inefficiency, the results of this lookup are memoized.
/// [`NamespaceTreeNode`]s are all stored in [`NamespaceTreeCell`]s, which are reference counted and support interior mutability for namespace
/// insertions and clone-free lookups.
#[derive(Clone)]
pub struct NamespaceTreeRoot {
    assigner: usize,
    tree: NamespaceTreeCell,
    memo: RefCell<FxHashMap<NamespaceId, MemoEntry>>,
}

impl std::fmt::Debug for NamespaceTreeRoot {
    // manual implementation to avoid infinite loops in printing
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "NamespaceTreeRoot\n{}",
            self.tree.borrow().debug_print(0, &mut FxHashSet::default())
        )
    }
}

impl std::fmt::Debug for NamespaceTreeNode {
    // manual implementation to avoid infinite loops in printing
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.debug_print(0, &mut FxHashSet::default()))
    }
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
    /// Panics if the `ns` iterator is empty.
    #[must_use]
    pub fn insert_or_find_namespace(
        &mut self,
        ns: impl IntoIterator<Item = Rc<str>>,
    ) -> NamespaceId {
        self.tree
            .borrow_mut()
            .insert_or_find_namespace(ns.into_iter().peekable(), &mut self.assigner)
            .expect("namespace creation should not fail")
    }

    /// Get the ID of a namespace given its name.
    pub fn get_namespace_id<'a>(
        &self,
        ns: impl IntoIterator<Item = &'a str>,
    ) -> Option<NamespaceId> {
        self.tree.borrow().get_namespace_id(ns)
    }

    /// Given a [`NamespaceId`], find the namespace in the tree. Note that this function is not
    /// particularly efficient, as it performs a breadth-first search. The results of this search
    /// are memoized to avoid repeated lookups, reducing the impact of the BFS.
    #[must_use]
    pub fn find_namespace_by_id(&self, id: &NamespaceId) -> (Vec<Rc<str>>, NamespaceTreeCell) {
        if let Some(res) = self.memo.borrow().get(id) {
            return res.clone();
        }
        let (names, node) = self
            .tree
            .borrow()
            .find_namespace_by_id(*id, &[], &mut FxHashSet::default())
            .unwrap_or_else(|| (vec![], self.tree.clone()));

        self.memo
            .borrow_mut()
            .insert(*id, (names.clone(), node.clone()));
        (names, node.clone())
    }

    #[must_use]
    pub fn root_id(&self) -> NamespaceId {
        self.tree.borrow().id
    }

    /// Inserts an alias for an existing namespace into the tree, where the child namespace already exists.
    pub fn insert_with_id(
        &mut self,
        parent: Option<NamespaceId>,
        new_child: NamespaceId,
        alias: &str,
    ) {
        let parent = parent.unwrap_or_else(|| self.root_id());
        let (_, parent_node) = self.find_namespace_by_id(&parent);
        let (_, existing_ns) = self.find_namespace_by_id(&new_child);
        parent_node
            .borrow_mut()
            .children
            .insert(Rc::from(alias), existing_ns);
    }

    /// Inserts (or finds) a new namespace as a child of an existing namespace.
    /// Primarily used for appending namespaces to a parent namespace which represents a module/external package..
    pub fn insert_or_find_namespace_from_root(
        &mut self,
        ns: impl Into<Vec<Rc<str>>>,
        root: NamespaceId,
    ) -> NamespaceId {
        let (_root_name, root_contents) = self.find_namespace_by_id(&root);
        let id = root_contents
            .borrow_mut()
            .insert_or_find_namespace(ns.into().into_iter().peekable(), &mut self.assigner);

        id.expect("empty name should not be passed into namespace insertion")
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
            let _ = tree.insert_or_find_namespace(iter);
        }
        tree
    }
}

/// A node in the namespace tree. Each node has a unique ID and a map of children.
/// Supports interior mutability of children for inserting new nodes.
#[derive(Clone)]
pub struct NamespaceTreeNode {
    pub children: FxHashMap<Rc<str>, NamespaceTreeCell>,
    pub id: NamespaceId,
}

impl NamespaceTreeNode {
    /// Create a new namespace tree node with the given ID and children. The `id` should come from the `NamespaceTreeRoot` assigner.
    #[must_use]
    fn new(id: NamespaceId, children: FxHashMap<Rc<str>, NamespaceTreeCell>) -> Self {
        Self { children, id }
    }

    /// Get a reference to the children of the namespace tree node.
    #[must_use]
    pub fn children(&self) -> &FxHashMap<Rc<str>, NamespaceTreeCell> {
        &self.children
    }

    /// See [`FxHashMap::get`] for more information.
    fn get(&self, component: &Rc<str>) -> Option<NamespaceTreeCell> {
        self.children.get(component).cloned()
    }

    /// Get the ID of this namespace tree node.
    #[must_use]
    pub fn id(&self) -> NamespaceId {
        self.id
    }

    /// Check if this namespace tree node contains a given namespace as a child.
    #[must_use]
    pub fn contains<'a>(&self, ns: impl IntoIterator<Item = &'a str>) -> bool {
        self.get_namespace_id(ns).is_some()
    }

    /// Finds the ID of a namespace given its string name. This function is generally more efficient
    /// than [`NamespaceTreeNode::find_namespace_by_id`], as it utilizes the prefix tree structure to
    /// find the ID in `O(n)` time, where `n` is the number of components in the namespace name.
    pub fn get_namespace_id<'a>(
        &self,
        ns: impl IntoIterator<Item = &'a str>,
    ) -> Option<NamespaceId> {
        let mut rover: Option<NamespaceTreeCell> = None;
        for component in ns {
            if let Some(next_ns) = match rover {
                None => self.get(&Rc::from(component)),
                Some(buf) => buf.borrow().get(&Rc::from(component)),
            } {
                rover = Some(next_ns);
            } else {
                return None;
            }
        }
        Some(rover.map_or_else(|| self.id, |x| x.borrow().id))
    }

    /// Inserts a new namespace into the tree, if it does not yet exist.
    /// Returns the ID of the namespace.
    /// Returns `None` if an empty iterator is passed in.
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

    fn find_namespace_by_id(
        &self,
        id: NamespaceId,
        names_buf: &[Rc<str>],
        // `ids_visited` is used to avoid infinite loops in the case of cycles in the namespace tree.
        ids_visited: &mut FxHashSet<NamespaceId>,
    ) -> Option<(Vec<Rc<str>>, NamespaceTreeCell)> {
        if ids_visited.contains(&self.id) {
            return None;
        }
        ids_visited.insert(self.id);
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
            let Some((names, node)) = node.borrow().find_namespace_by_id(id, &names, ids_visited)
            else {
                continue;
            };
            return Some((names, node));
        }

        None
    }

    fn debug_print(
        &self,
        indentation_level: usize,
        visited_nodes: &mut FxHashSet<NamespaceId>,
    ) -> String {
        let indentation = "  ".repeat(indentation_level);

        if visited_nodes.contains(&self.id) {
            return format!("\n{indentation}Cycle Detected");
        }

        visited_nodes.insert(self.id);

        let mut result = String::new();

        if self.children.is_empty() {
            result.push_str("empty node");
        } else {
            result.push_str(&format!("\n{indentation}  children: ["));
            for (name, node) in &self.children {
                result.push_str(&format!(
                    "\n{}    {}(id {}) {{",
                    indentation,
                    name,
                    Into::<usize>::into(node.borrow().id)
                ));
                result.push_str(
                    node.borrow()
                        .debug_print(indentation_level + 2, visited_nodes)
                        .as_str(),
                );
                result.push(',');
            }
            result.push_str(&format!("\n{indentation}  ]"));
            result.push_str(&format!("\n{indentation}"));
        }
        result.push('}');
        result
    }
}
