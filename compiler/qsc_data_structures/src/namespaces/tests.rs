use expect_test::expect;

use super::*;
use std::collections::HashMap;

#[test]
fn test_tree_construction() {
    let mut root = NamespaceTreeRoot::default();
    for i in 0..10 {
        for j in 'a'..'d' {
            root.insert_or_find_namespace(
                vec![Rc::from(format!("ns{}", i)), Rc::from(format!("ns{}", j))].into_iter(),
            );
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
            id_buf.push(root.insert_or_find_namespace(
                vec![Rc::from(format!("ns{}", i)), Rc::from(format!("ns{}", j))].into_iter(),
            ));
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
            "#]]
    .assert_debug_eq(&result_buf)
}
// test that after inserting lots of namespaces, all ids are unique and sequential
#[test]
fn test_insert_or_find_namespace() {
    let mut root = NamespaceTreeRoot::default();
    let mut ids: Vec<usize> = vec![];
    for i in 0..10 {
        for j in 'a'..'d' {
            let id = root.insert_or_find_namespace(
                vec![Rc::from(format!("ns{}", i)), Rc::from(format!("ns{}", j))].into_iter(),
            );
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
