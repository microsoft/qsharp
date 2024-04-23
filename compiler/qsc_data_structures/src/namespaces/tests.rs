use expect_test::expect;

use super::*;

#[allow(clippy::too_many_lines)]
#[test]
fn test_tree_construction() {
    let mut root = NamespaceTreeRoot::default();
    for i in 0..3 {
        for j in 'a'..'d' {
            root.insert_or_find_namespace(
                vec![Rc::from(format!("ns{i}")), Rc::from(format!("ns{j}"))].into_iter(),
            );
        }
    }
    expect![[r#"
        NamespaceTreeRoot {
            assigner: 18,
            tree: RefCell {
                value: NamespaceTreeNode {
                    children: {
                        "ns1": RefCell {
                            value: NamespaceTreeNode {
                                children: {
                                    "nsc": RefCell {
                                        value: NamespaceTreeNode {
                                            children: {},
                                            id: NamespaceId(
                                                14,
                                            ),
                                        },
                                    },
                                    "nsb": RefCell {
                                        value: NamespaceTreeNode {
                                            children: {},
                                            id: NamespaceId(
                                                13,
                                            ),
                                        },
                                    },
                                    "nsa": RefCell {
                                        value: NamespaceTreeNode {
                                            children: {},
                                            id: NamespaceId(
                                                12,
                                            ),
                                        },
                                    },
                                },
                                id: NamespaceId(
                                    11,
                                ),
                            },
                        },
                        "ns0": RefCell {
                            value: NamespaceTreeNode {
                                children: {
                                    "nsc": RefCell {
                                        value: NamespaceTreeNode {
                                            children: {},
                                            id: NamespaceId(
                                                10,
                                            ),
                                        },
                                    },
                                    "nsb": RefCell {
                                        value: NamespaceTreeNode {
                                            children: {},
                                            id: NamespaceId(
                                                9,
                                            ),
                                        },
                                    },
                                    "nsa": RefCell {
                                        value: NamespaceTreeNode {
                                            children: {},
                                            id: NamespaceId(
                                                8,
                                            ),
                                        },
                                    },
                                },
                                id: NamespaceId(
                                    7,
                                ),
                            },
                        },
                        "Microsoft": RefCell {
                            value: NamespaceTreeNode {
                                children: {
                                    "Quantum": RefCell {
                                        value: NamespaceTreeNode {
                                            children: {
                                                "Canon": RefCell {
                                                    value: NamespaceTreeNode {
                                                        children: {},
                                                        id: NamespaceId(
                                                            3,
                                                        ),
                                                    },
                                                },
                                                "Measurement": RefCell {
                                                    value: NamespaceTreeNode {
                                                        children: {},
                                                        id: NamespaceId(
                                                            6,
                                                        ),
                                                    },
                                                },
                                                "Core": RefCell {
                                                    value: NamespaceTreeNode {
                                                        children: {},
                                                        id: NamespaceId(
                                                            4,
                                                        ),
                                                    },
                                                },
                                                "Intrinsic": RefCell {
                                                    value: NamespaceTreeNode {
                                                        children: {},
                                                        id: NamespaceId(
                                                            5,
                                                        ),
                                                    },
                                                },
                                            },
                                            id: NamespaceId(
                                                2,
                                            ),
                                        },
                                    },
                                },
                                id: NamespaceId(
                                    1,
                                ),
                            },
                        },
                        "ns2": RefCell {
                            value: NamespaceTreeNode {
                                children: {
                                    "nsc": RefCell {
                                        value: NamespaceTreeNode {
                                            children: {},
                                            id: NamespaceId(
                                                18,
                                            ),
                                        },
                                    },
                                    "nsb": RefCell {
                                        value: NamespaceTreeNode {
                                            children: {},
                                            id: NamespaceId(
                                                17,
                                            ),
                                        },
                                    },
                                    "nsa": RefCell {
                                        value: NamespaceTreeNode {
                                            children: {},
                                            id: NamespaceId(
                                                16,
                                            ),
                                        },
                                    },
                                },
                                id: NamespaceId(
                                    15,
                                ),
                            },
                        },
                    },
                    id: NamespaceId(
                        0,
                    ),
                },
            },
            memo: RefCell {
                value: {},
            },
        }
    "#]]
    .assert_debug_eq(&root);
}

#[allow(clippy::too_many_lines)]
#[test]
fn test_find_id() {
    let mut root = NamespaceTreeRoot::default();
    let mut id_buf = vec![];
    for i in 0..3 {
        for j in 'a'..'d' {
            id_buf.push(root.insert_or_find_namespace(
                vec![Rc::from(format!("ns{i}")), Rc::from(format!("ns{j}"))].into_iter(),
            ));
        }
    }
    let mut result_buf = vec![];
    for id in id_buf {
        result_buf.push(root.find_namespace_by_id(&id));
    }
    expect![[r#"
        [
            (
                [
                    "ns0",
                    "nsa",
                ],
                RefCell {
                    value: NamespaceTreeNode {
                        children: {},
                        id: NamespaceId(
                            8,
                        ),
                    },
                },
            ),
            (
                [
                    "ns0",
                    "nsb",
                ],
                RefCell {
                    value: NamespaceTreeNode {
                        children: {},
                        id: NamespaceId(
                            9,
                        ),
                    },
                },
            ),
            (
                [
                    "ns0",
                    "nsc",
                ],
                RefCell {
                    value: NamespaceTreeNode {
                        children: {},
                        id: NamespaceId(
                            10,
                        ),
                    },
                },
            ),
            (
                [
                    "ns1",
                    "nsa",
                ],
                RefCell {
                    value: NamespaceTreeNode {
                        children: {},
                        id: NamespaceId(
                            12,
                        ),
                    },
                },
            ),
            (
                [
                    "ns1",
                    "nsb",
                ],
                RefCell {
                    value: NamespaceTreeNode {
                        children: {},
                        id: NamespaceId(
                            13,
                        ),
                    },
                },
            ),
            (
                [
                    "ns1",
                    "nsc",
                ],
                RefCell {
                    value: NamespaceTreeNode {
                        children: {},
                        id: NamespaceId(
                            14,
                        ),
                    },
                },
            ),
            (
                [
                    "ns2",
                    "nsa",
                ],
                RefCell {
                    value: NamespaceTreeNode {
                        children: {},
                        id: NamespaceId(
                            16,
                        ),
                    },
                },
            ),
            (
                [
                    "ns2",
                    "nsb",
                ],
                RefCell {
                    value: NamespaceTreeNode {
                        children: {},
                        id: NamespaceId(
                            17,
                        ),
                    },
                },
            ),
            (
                [
                    "ns2",
                    "nsc",
                ],
                RefCell {
                    value: NamespaceTreeNode {
                        children: {},
                        id: NamespaceId(
                            18,
                        ),
                    },
                },
            ),
        ]
    "#]]
    .assert_debug_eq(&result_buf);
}
// test that after inserting lots of namespaces, all ids are unique and sequential
#[test]
fn test_insert_or_find_namespace() {
    let mut root = NamespaceTreeRoot::default();
    let mut ids: Vec<usize> = vec![];
    for i in 0..3 {
        for j in 'a'..'d' {
            let id = root.insert_or_find_namespace(
                vec![Rc::from(format!("ns{i}")), Rc::from(format!("ns{j}"))].into_iter(),
            );
            ids.push(id.into());
        }
    }
    let mut ids_sorted = ids.clone();
    ids_sorted.sort_unstable();
    ids_sorted.dedup();
    // there should be no duplicate or out-of-order ids
    assert_eq!(ids_sorted, ids);
    expect![[r"
        [
            8,
            9,
            10,
            12,
            13,
            14,
            16,
            17,
            18,
        ]
    "]]
    .assert_debug_eq(&ids);
}
