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
            let _ = root.insert_or_find_namespace(
                vec![Rc::from(format!("ns{i}")), Rc::from(format!("ns{j}"))].into_iter(),
            );
        }
    }
    expect![[r#"
        NamespaceTreeRoot

          children: [
            Std(id 1) {
              children: [
                Measurement(id 7) {empty node},
                Canon(id 2) {empty node},
                Intrinsic(id 6) {empty node},
              ]
            },
            ns1(id 12) {
              children: [
                nsc(id 15) {empty node},
                nsb(id 14) {empty node},
                nsa(id 13) {empty node},
              ]
            },
            ns0(id 8) {
              children: [
                nsc(id 11) {empty node},
                nsb(id 10) {empty node},
                nsa(id 9) {empty node},
              ]
            },
            Microsoft(id 3) {
              children: [
                Quantum(id 4) {
                  children: [
                    Core(id 5) {empty node},
                  ]
                },
              ]
            },
            ns2(id 16) {
              children: [
                nsc(id 19) {empty node},
                nsb(id 18) {empty node},
                nsa(id 17) {empty node},
              ]
            },
          ]
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
                    value: empty node},
                },
            ),
            (
                [
                    "ns0",
                    "nsb",
                ],
                RefCell {
                    value: empty node},
                },
            ),
            (
                [
                    "ns0",
                    "nsc",
                ],
                RefCell {
                    value: empty node},
                },
            ),
            (
                [
                    "ns0",
                ],
                RefCell {
                    value: 
                      children: [
                        nsc(id 11) {empty node},
                        nsb(id 10) {empty node},
                        nsa(id 9) {empty node},
                      ]
                    },
                },
            ),
            (
                [
                    "ns1",
                    "nsa",
                ],
                RefCell {
                    value: empty node},
                },
            ),
            (
                [
                    "ns1",
                    "nsb",
                ],
                RefCell {
                    value: empty node},
                },
            ),
            (
                [
                    "ns1",
                    "nsc",
                ],
                RefCell {
                    value: empty node},
                },
            ),
            (
                [
                    "ns1",
                ],
                RefCell {
                    value: 
                      children: [
                        nsc(id 15) {empty node},
                        nsb(id 14) {empty node},
                        nsa(id 13) {empty node},
                      ]
                    },
                },
            ),
            (
                [
                    "ns2",
                    "nsa",
                ],
                RefCell {
                    value: empty node},
                },
            ),
            (
                [
                    "ns2",
                    "nsb",
                ],
                RefCell {
                    value: empty node},
                },
            ),
            (
                [
                    "ns2",
                    "nsc",
                ],
                RefCell {
                    value: empty node},
                },
            ),
            (
                [
                    "ns2",
                ],
                RefCell {
                    value: 
                      children: [
                        nsc(id 19) {empty node},
                        nsb(id 18) {empty node},
                        nsa(id 17) {empty node},
                      ]
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
    expect![[r#"
        [
            9,
            10,
            11,
            13,
            14,
            15,
            17,
            18,
            19,
        ]
    "#]]
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
            let _ = root.insert_or_find_namespace(name.clone());
            names_to_query_buf.push(name);
        }
        let name = vec![Rc::from(format!("ns{i}"))];
        let _ = root.insert_or_find_namespace(name.clone());
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
                    11,
                ),
            ),
            Some(
                NamespaceId(
                    8,
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
                    15,
                ),
            ),
            Some(
                NamespaceId(
                    12,
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
                    19,
                ),
            ),
            Some(
                NamespaceId(
                    16,
                ),
            ),
        ]
    "#]]
    .assert_debug_eq(&result_buf);
}

#[allow(clippy::too_many_lines)]
#[test]
fn test_tree_iter() {
    let mut root = NamespaceTreeRoot::default();
    for i in 0..3 {
        for j in 'a'..'d' {
            let _ = root.insert_or_find_namespace(
                vec![Rc::from(format!("ns{i}")), Rc::from(format!("ns{j}"))].into_iter(),
            );
        }
    }

    let result = root.iter().collect::<Vec<_>>();
    expect![[r#"
        [
            [
                [],
            ],
            [
                [
                    "Std",
                ],
            ],
            [
                [
                    "Std",
                    "Canon",
                ],
                [
                    "Std",
                    "Canon",
                ],
            ],
            [
                [
                    "Microsoft",
                ],
            ],
            [
                [
                    "Microsoft",
                    "Quantum",
                ],
            ],
            [
                [
                    "Microsoft",
                    "Quantum",
                    "Core",
                ],
                [
                    "Microsoft",
                    "Quantum",
                    "Core",
                ],
            ],
            [
                [
                    "Std",
                    "Intrinsic",
                ],
                [
                    "Std",
                    "Intrinsic",
                ],
            ],
            [
                [
                    "Std",
                    "Measurement",
                ],
                [
                    "Std",
                    "Measurement",
                ],
            ],
            [
                [
                    "ns0",
                ],
            ],
            [
                [
                    "ns0",
                    "nsa",
                ],
                [
                    "ns0",
                    "nsa",
                ],
            ],
            [
                [
                    "ns0",
                    "nsb",
                ],
                [
                    "ns0",
                    "nsb",
                ],
            ],
            [
                [
                    "ns0",
                    "nsc",
                ],
                [
                    "ns0",
                    "nsc",
                ],
            ],
            [
                [
                    "ns1",
                ],
            ],
            [
                [
                    "ns1",
                    "nsa",
                ],
                [
                    "ns1",
                    "nsa",
                ],
            ],
            [
                [
                    "ns1",
                    "nsb",
                ],
                [
                    "ns1",
                    "nsb",
                ],
            ],
            [
                [
                    "ns1",
                    "nsc",
                ],
                [
                    "ns1",
                    "nsc",
                ],
            ],
            [
                [
                    "ns2",
                ],
            ],
            [
                [
                    "ns2",
                    "nsa",
                ],
                [
                    "ns2",
                    "nsa",
                ],
            ],
            [
                [
                    "ns2",
                    "nsb",
                ],
                [
                    "ns2",
                    "nsb",
                ],
            ],
            [
                [
                    "ns2",
                    "nsc",
                ],
                [
                    "ns2",
                    "nsc",
                ],
            ],
        ]
    "#]]
    .assert_debug_eq(&result);
}
