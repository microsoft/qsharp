// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Point, Point2D, Population};

#[test]
fn test_dominates_should_dominate_by_first_dimension() {
    let p = Point2D::new(0, 1.0, 1);
    let other = Point2D::new(1, 1.5, 1);
    assert!(p.dominates(&other));
}

#[test]
fn test_dominates_should_dominate_by_second_dimension() {
    let p = Point2D::new(0, 1.0, 1);
    let other = Point2D::new(1, 1.0, 5);
    assert!(p.dominates(&other));
}

#[test]
fn test_dominates_should_dominate_by_both_dimensions() {
    let p = Point2D::new(0, 1.0, 1);
    let other = Point2D::new(1, 2.0, 5);
    assert!(p.dominates(&other));
}

#[test]
fn test_dominates_should_not_dominate_first_less_second_greater() {
    let p = Point2D::new(0, 1.0, 1);
    let other = Point2D::new(1, 3.0, 0);
    assert!(!p.dominates(&other));
}

#[test]
fn test_dominates_should_not_dominate_first_greater_second_less() {
    let p = Point2D::new(0, 1.0, 1);
    let other = Point2D::new(1, 0.0, 2);
    assert!(!p.dominates(&other));
}

#[test]
fn test_dominates_should_not_dominate_itself() {
    let p = Point2D::new(0, 1.0, 1);
    let other = Point2D::new(1, 1.0, 1);
    assert!(!p.dominates(&other));
}

#[test]
fn fill() {
    let mut population = Population::<Point2D<i32>>::new();
    let p1 = Point2D::new(0, 1.0, 1);
    let p2 = Point2D::new(1, 0.5, 1);
    let p3 = Point2D::new(2, 1.1, 0);
    population.push(p1);
    population.push(p2);
    population.push(p3);
    let items = population.items();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].item, 0);
    assert_eq!(items[1].item, 1);
    assert_eq!(items[2].item, 2);
}

#[test]
fn fill_and_filter() {
    let mut population = Population::<Point2D<i32>>::new();
    let p1 = Point2D::new(0, 1.0, 1);
    let p2 = Point2D::new(1, 0.5, 1);
    let p3 = Point2D::new(2, 1.1, 0);
    population.push(p1);
    population.push(p2);
    population.push(p3);
    population.filter_out_dominated();
    let items = population.items();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].item + items[1].item, 3);
}

#[test]
fn fill_filter_and_sort() {
    let mut population = Population::<Point2D<i32>>::new();
    let p1 = Point2D::new(0, 1.0, 1);
    let p2 = Point2D::new(1, 0.5, 1);
    let p3 = Point2D::new(2, 1.1, 0);
    population.push(p1);
    population.push(p2);
    population.push(p3);
    population.filter_out_dominated();
    population.sort_items();
    let items = population.items();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].item, 2);
    assert_eq!(items[1].item, 1);
}
