// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::hir::{self, Item, ItemId, ItemKind, Package, PackageId, VisibilityKind};
use qsc_data_structures::index_map;
use std::{collections::HashMap, rc::Rc};

pub struct Global {
    pub namespace: Rc<str>,
    pub name: Rc<str>,
    pub visibility: VisibilityKind,
    pub kind: Kind,
}

pub enum Kind {
    Ty(Ty),
    Term(Term),
}

pub struct Ty {
    pub id: ItemId,
}

pub struct Term {
    pub id: ItemId,
    pub ty: hir::Ty,
}

#[derive(Default)]
pub struct Table {
    tys: HashMap<Rc<str>, HashMap<Rc<str>, Ty>>,
    terms: HashMap<Rc<str>, HashMap<Rc<str>, Term>>,
}

impl Table {
    #[must_use]
    pub fn resolve_ty(&self, namespace: &str, name: &str) -> Option<&Ty> {
        self.tys.get(namespace).and_then(|terms| terms.get(name))
    }

    #[must_use]
    pub fn resolve_term(&self, namespace: &str, name: &str) -> Option<&Term> {
        self.terms.get(namespace).and_then(|terms| terms.get(name))
    }
}

impl FromIterator<Global> for Table {
    fn from_iter<T: IntoIterator<Item = Global>>(iter: T) -> Self {
        let mut tys: HashMap<_, HashMap<_, _>> = HashMap::new();
        let mut terms: HashMap<_, HashMap<_, _>> = HashMap::new();
        for global in iter {
            match global.kind {
                Kind::Ty(ty) => {
                    tys.entry(global.namespace)
                        .or_default()
                        .insert(global.name, ty);
                }
                Kind::Term(term) => {
                    terms
                        .entry(global.namespace)
                        .or_default()
                        .insert(global.name, term);
                }
            }
        }

        Self { tys, terms }
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
        let parent = self
            .package
            .items
            .get(item.parent?)
            .expect("parent should exist");

        let ItemKind::Namespace(namespace, _) = &parent.kind else { return None; };
        let visibility = item.visibility.map_or(VisibilityKind::Public, |v| v.kind);
        let id = ItemId {
            package: self.id,
            item: item.id,
        };

        match &item.kind {
            ItemKind::Callable(decl) => Some(Global {
                namespace: Rc::clone(&namespace.name),
                name: Rc::clone(&decl.name.name),
                visibility,
                kind: Kind::Term(Term { id, ty: decl.ty() }),
            }),
            ItemKind::Ty(name, def) => {
                self.next = Some(Global {
                    namespace: Rc::clone(&namespace.name),
                    name: Rc::clone(&name.name),
                    visibility,
                    kind: Kind::Term(Term {
                        id,
                        ty: def.cons_ty(id),
                    }),
                });

                Some(Global {
                    namespace: Rc::clone(&namespace.name),
                    name: Rc::clone(&name.name),
                    visibility,
                    kind: Kind::Ty(Ty { id }),
                })
            }
            ItemKind::Namespace(..) => None,
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
