// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// allowing needless raw hashes here because we auto-update these expected outputs
// and don't want to risk weird breakages

#![allow(clippy::needless_raw_string_hashes)]

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
        id_buf.push(root.insert_or_find_namespace(vec![Rc::from(format!("ns{i}"))].into_iter()));
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
                    "ns0",
                ],
                RefCell {
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
                    "ns1",
                ],
                RefCell {
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
            (
                [
                    "ns2",
                ],
                RefCell {
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

// test for get_namespace_id
#[test]
fn test_get_namespace_id() {
    let mut root = NamespaceTreeRoot::default();
    let mut names_to_query_buf = vec![];
    for i in 0..3 {
        for j in 'a'..'d' {
            let name = vec![Rc::from(format!("ns{i}")), Rc::from(format!("ns{j}"))];
            root.insert_or_find_namespace(name.clone());
            names_to_query_buf.push(name);
        }
        let name = vec![Rc::from(format!("ns{i}"))];
        root.insert_or_find_namespace(name.clone());
        names_to_query_buf.push(name);
    }
    let mut result_buf = vec![];
    for name in names_to_query_buf {
        result_buf.push(root.get_namespace_id(name.iter().map(|x| &**x)));
    }
    expect![[r#"
        [
            Some(
                NamespaceId(
                    8,
                ),
            ),
            Some(
                NamespaceId(
                    9,
                ),
            ),
            Some(
                NamespaceId(
                    10,
                ),
            ),
            Some(
                NamespaceId(
                    7,
                ),
            ),
            Some(
                NamespaceId(
                    12,
                ),
            ),
            Some(
                NamespaceId(
                    13,
                ),
            ),
            Some(
                NamespaceId(
                    14,
                ),
            ),
            Some(
                NamespaceId(
                    11,
                ),
            ),
            Some(
                NamespaceId(
                    16,
                ),
            ),
            Some(
                NamespaceId(
                    17,
                ),
            ),
            Some(
                NamespaceId(
                    18,
                ),
            ),
            Some(
                NamespaceId(
                    15,
                ),
            ),
        ]
    "#]]
    .assert_debug_eq(&result_buf);
}
