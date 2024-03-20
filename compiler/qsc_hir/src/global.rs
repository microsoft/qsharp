// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    hir::{Ident, Item, ItemId, ItemKind, ItemStatus, Package, PackageId, SpecBody, SpecGen, VecIdent, Visibility},
    ty::Scheme,
};
use qsc_data_structures::index_map;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Global {
    pub namespace: Vec<Rc<str>>,
    pub name: Rc<str>,
    pub visibility: Visibility,
    pub status: ItemStatus,
    pub kind: Kind,
}

pub enum Kind {
    Namespace,
    Ty(Ty),
    Term(Term),
}

pub struct Ty {
    pub id: ItemId,
}

pub struct Term {
    pub id: ItemId,
    pub scheme: Scheme,
    pub intrinsic: bool,
}


/// A lookup table used for looking up global core items for insertion in `qsc_passes`.
#[derive(Default)]
pub struct Table {
    tys: FxHashMap<NamespaceId, FxHashMap<Rc<str>, Ty>>,
    terms: FxHashMap<NamespaceId, FxHashMap<Rc<str>, Term>>,
    namespaces: NamespaceTreeRoot
}

impl Table {
    #[must_use]
    pub fn resolve_ty(&self, namespace: NamespaceId, name: &str) -> Option<&Ty> {
        self.tys.get(&namespace).and_then(|terms| terms.get(name))
    }

    #[must_use]
    pub fn resolve_term(&self, namespace: NamespaceId, name: &str) -> Option<&Term> {
        self.terms.get(&namespace).and_then(|terms| terms.get(name))
    }
}

impl FromIterator<Global> for Table {
    fn from_iter<T: IntoIterator<Item = Global>>(iter: T) -> Self {
        let mut tys: FxHashMap<_, FxHashMap<_, _>> = FxHashMap::default();
        let mut terms: FxHashMap<_, FxHashMap<_, _>> = FxHashMap::default();
        let mut namespaces = NamespaceTreeRoot::default();
        for global in iter {
            let namespace = namespaces.upsert_namespace(global.namespace);
            match global.kind {
                Kind::Ty(ty) => {
                    tys.entry(namespace)
                        .or_default()
                        .insert(global.name, ty);
                }
                Kind::Term(term) => {
                    terms
                        .entry(namespace)
                        .or_default()
                        .insert(global.name, term);
                }
                Kind::Namespace => {}
            }
        }

        // TODO; copy namespace root etc over here and 
        // create a namespace structure with IDs
        Self { namespaces, tys, terms }
    }
}

pub struct NamespaceTreeRoot {
    assigner: usize,
    tree: NamespaceTreeNode,
}

impl NamespaceTreeRoot {
    fn upsert_namespace(&mut self, name: impl Into<Vec<Rc<str>>>) -> NamespaceId {
        self.assigner += 1;
        let id = self.assigner;
        let node = self.new_namespace_node(Default::default());
        let mut components_iter = name.into();
        let mut components_iter = components_iter.iter();
        // construct the initial rover for the breadth-first insertion
        // (this is like a BFS but we create a new node if one doesn't exist)
        let self_cell = RefCell::new(self);
        let mut rover_node = &mut self_cell.borrow_mut().tree;
        // create the rest of the nodes
        for component in components_iter {
            rover_node = rover_node.children
                .entry(Rc::clone(component))
                .or_insert_with(|| self_cell.borrow_mut().new_namespace_node(Default::default()));
        }

        rover_node.id
    }
    fn new_namespace_node(
        &mut self,
        children: HashMap<Rc<str>, NamespaceTreeNode>,
    ) -> NamespaceTreeNode {
        self.assigner += 1;
        NamespaceTreeNode {
            id: NamespaceId::new(self.assigner),
            children,
        }
    }

    fn find_namespace(&self, ns: &VecIdent) -> Option<NamespaceId> {
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
    children: HashMap<Rc<str>, NamespaceTreeNode>,
    id: NamespaceId,
}
impl NamespaceTreeNode {
    fn get(&self, component: &Ident) -> Option<&NamespaceTreeNode> {
        self.children.get(&component.name)
    }

    fn contains(&self, ns: &VecIdent) -> bool {
        self.find_namespace(ns).is_some()
    }
    fn find_namespace(&self, ns: &VecIdent) -> Option<NamespaceId> {
        // look up a namespace in the tree and return the id
        // do a breadth-first search through NamespaceTree for the namespace
        // if it's not found, return None
        let mut buf = Rc::new(self);
        for component in ns.iter() {
            if let Some(next_ns) = buf.get(component) {
                buf = Rc::new(next_ns);
            } else {
                return None;
            }
        }
        return Some(buf.id);
    }
}

/// An ID that corresponds to a namespace in the global scope.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Default)]
pub struct NamespaceId(usize);
impl NamespaceId {
    pub fn new(value: usize) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for NamespaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Namespace {}", self.0)
    }
}
pub struct PackageIter<'a> {
    id: Option<PackageId>,
    package: &'a Package,
    items: index_map::Values<'a, Item>,
    next: Option<Global>,
}

impl PackageIter<'_> {
    fn global_item(&mut self, item: &Item) -> Option<Global> {
        let parent = item.parent.map(|parent| {
            &self
                .package
                .items
                .get(parent)
                .expect("parent should exist")
                .kind
        });
        let id = ItemId {
            package: self.id,
            item: item.id,
        };
        let status = ItemStatus::from_attrs(item.attrs.as_ref());

        match (&item.kind, &parent) {
            (ItemKind::Callable(decl), Some(ItemKind::Namespace(namespace, _))) => Some(Global {
                namespace: namespace.into(),
                name: Rc::clone(&decl.name.name),
                visibility: item.visibility,
                status,
                kind: Kind::Term(Term {
                    id,
                    scheme: decl.scheme(),
                    intrinsic: decl.body.body == SpecBody::Gen(SpecGen::Intrinsic),
                }),
            }),
            (ItemKind::Ty(name, def), Some(ItemKind::Namespace(namespace, _))) => {
                self.next = Some(Global {
                    namespace: namespace.into(),
                    name: Rc::clone(&name.name),
                    visibility: item.visibility,
                    status,
                    kind: Kind::Term(Term {
                        id,
                        scheme: def.cons_scheme(id),
                        intrinsic: false,
                    }),
                });

                Some(Global {
                    namespace: namespace.into(),
                    name: Rc::clone(&name.name),
                    visibility: item.visibility,
                    status,
                    kind: Kind::Ty(Ty { id }),
                })
            }
            (ItemKind::Namespace(ident, _), None) => Some(Global {
                namespace:  ident.into(),
                name: "".into(),
                visibility: Visibility::Public,
                status,
                kind: Kind::Namespace,
            }),
            _ => None,
        }
    }
}

impl<'a> Iterator for PackageIter<'a> {
    type Item = Global;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(global) = self.next.take() {
            Some(global)
        } else {
            loop {
                let item = self.items.next()?;
                if let Some(global) = self.global_item(item) {
                    break Some(global);
                }
            }
        }
    }
}

#[must_use]
pub fn iter_package(id: Option<PackageId>, package: &Package) -> PackageIter {
    PackageIter {
        id,
        package,
        items: package.items.values(),
        next: None,
    }
}
