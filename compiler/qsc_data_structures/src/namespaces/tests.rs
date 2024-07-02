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
            ns1(id 11) {
              children: [
                nsc(id 14) {empty node},
                nsb(id 13) {empty node},
                nsa(id 12) {empty node},
              ]
            },
            ns0(id 7) {
              children: [
                nsc(id 10) {empty node},
                nsb(id 9) {empty node},
                nsa(id 8) {empty node},
              ]
            },
            Microsoft(id 1) {
              children: [
                Quantum(id 2) {
                  children: [
                    Canon(id 3) {empty node},
                    Measurement(id 6) {empty node},
                    Core(id 4) {empty node},
                    Intrinsic(id 5) {empty node},
                  ]
                },
              ]
            },
            ns2(id 15) {
              children: [
                nsc(id 18) {empty node},
                nsb(id 17) {empty node},
                nsa(id 16) {empty node},
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
                        nsc(id 10) {empty node},
                        nsb(id 9) {empty node},
                        nsa(id 8) {empty node},
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
                        nsc(id 14) {empty node},
                        nsb(id 13) {empty node},
                        nsa(id 12) {empty node},
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
                        nsc(id 18) {empty node},
                        nsb(id 17) {empty node},
                        nsa(id 16) {empty node},
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
