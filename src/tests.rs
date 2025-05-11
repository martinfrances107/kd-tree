#![cfg(test)]
use super::*;

#[test]
fn test_nearest() {
    let mut gen3d = random3d_generator();
    let kdtree = KdTree::build_by_ordered_float(vec(10000, |_| gen3d()));
    for _ in 0..100 {
        let query = gen3d();
        let found = kdtree.nearest(&query).unwrap().item;
        let expected = kdtree
            .iter()
            .min_by_key(|p| ordered_float::OrderedFloat(squared_distance(p, &query)))
            .unwrap();
        assert_eq!(found, expected);
    }
}

#[test]
fn test_nearests() {
    test_nearests_by(random3d_generator());
    test_nearests_by(random3d_10th_generator());
}

fn test_nearests_by(mut gen3d: impl FnMut() -> [f64; 3]) {
    let kdtree = KdTree::build_by_ordered_float(vec(10000, |_| gen3d()));
    const NUM: usize = 5;
    for _ in 0..100 {
        let query = gen3d();
        let neighborhood = kdtree.nearests(&query, NUM);
        assert_eq!(neighborhood.len(), NUM);
        for i in 1..neighborhood.len() {
            assert!(neighborhood[i - 1].squared_distance <= neighborhood[i].squared_distance);
        }
        let neighborhood_radius = neighborhood
            .iter()
            .max_by_key(|entry| ordered_float::OrderedFloat(entry.squared_distance))
            .unwrap()
            .squared_distance;
        let neighborhood_contains = |p: &[f64; 3]| {
            neighborhood
                .iter()
                .any(|entry| std::ptr::eq(entry.item as _, p as _))
        };
        assert!(kdtree.iter().all(
            |p| neighborhood_contains(p) || neighborhood_radius <= squared_distance(p, &query)
        ));
    }
}

#[test]
fn test_within() {
    let mut gen3d = random3d_generator();
    let kdtree = KdTree::build_by_ordered_float(vec(10000, |_| gen3d()));
    for _ in 0..100 {
        let mut p1 = gen3d();
        let mut p2 = gen3d();
        for k in 0..3 {
            if p1[k] > p2[k] {
                std::mem::swap(&mut p1[k], &mut p2[k]);
            }
        }
        let found = kdtree.within(&[p1, p2]);
        let count = kdtree
            .iter()
            .filter(|p| (0..3).all(|k| p1[k] <= p[k] && p[k] <= p2[k]))
            .count();
        assert_eq!(found.len(), count);
    }
}

#[test]
fn test_within_against_empty() {
    let empty: KdTree<[f64; 3]> = KdTree::build_by_ordered_float(vec![]);
    assert!(empty.within(&[[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]).is_empty());
}

#[test]
fn test_within_radius() {
    let mut gen3d = random3d_generator();
    let kdtree = KdTree::build_by_ordered_float(vec(10000, |_| gen3d()));
    const RADIUS: f64 = 0.1;
    for _ in 0..100 {
        let query = gen3d();
        let found = kdtree.within_radius(&query, RADIUS);
        let count = kdtree
            .iter()
            .filter(|p| squared_distance(p, &query) < RADIUS * RADIUS)
            .count();
        assert_eq!(found.len(), count);
    }
}

fn squared_distance<T: num_traits::Num + Copy>(p1: &[T; 3], p2: &[T; 3]) -> T {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    let dz = p1[2] - p2[2];
    dx * dx + dy * dy + dz * dz
}

fn random3d_generator() -> impl FnMut() -> [f64; 3] {
    use rand::Rng;
    let mut rng = rand::rng();
    move || [rng.random(), rng.random(), rng.random()]
}

fn random3d_10th_generator() -> impl FnMut() -> [f64; 3] {
    // generates a random number between 0 and 1 with 0.1 step
    fn random_10th(rng: &mut impl rand::Rng) -> f64 {
        f64::from(rng.random_range(0u8..=10u8)) / 10.0
    }
    let mut rng = rand::rng();
    move || {
        [
            random_10th(&mut rng),
            random_10th(&mut rng),
            random_10th(&mut rng),
        ]
    }
}

fn vec<T>(count: usize, mut f: impl FnMut(usize) -> T) -> Vec<T> {
    let mut items = Vec::with_capacity(count);
    for i in 0..count {
        items.push(f(i));
    }
    items
}

#[cfg(feature = "nalgebra")]
#[test]
fn test_nalgebra_point() {
    use ::nalgebra as na;

    let mut gen3d = random3d_generator();
    let kdtree: KdTree<na::Point3<f64>> =
        KdTree::build_by_ordered_float(vec(10000, |_| gen3d().into()));
    for _ in 0..100 {
        let query: na::Point3<f64> = gen3d().into();
        let found = kdtree.nearest(&query).unwrap().item;
        let expected = kdtree
            .iter()
            .min_by_key(|p| ordered_float::OrderedFloat((query - **p).norm()))
            .unwrap();
        assert_eq!(found, expected);
    }
}

#[cfg(feature = "nalgebra")]
#[test]
fn test_nalgebra_vector() {
    use ::nalgebra as na;

    let mut gen3d = random3d_generator();
    let kdtree: KdTree<na::Vector3<f64>> =
        KdTree::build_by_ordered_float(vec(10000, |_| gen3d().into()));
    for _ in 0..100 {
        let query: na::Vector3<f64> = gen3d().into();
        let found = kdtree.nearest(&query).unwrap().item;
        let expected = kdtree
            .iter()
            .min_by_key(|p| ordered_float::OrderedFloat((query - **p).norm()))
            .unwrap();
        assert_eq!(found, expected);
    }
}

#[cfg(feature = "serde")]
#[test]
fn test_serde() {
    let src: KdTree3<[i32; 3]> = KdTree::build(vec![[1, 2, 3], [4, 5, 6]]);

    let json = serde_json::to_string(&src).unwrap();
    assert_eq!(json, "[[1,2,3],[4,5,6]]");

    let dst: KdTree3<[i32; 3]> = serde_json::from_str(&json).unwrap();
    assert_eq!(src, dst);
}

#[cfg(feature = "nalgebra-serde")]
#[test]
fn test_nalgebra_serde() {
    use ::nalgebra as na;

    let src: KdTree<na::Point3<f64>> = KdTree::build_by_ordered_float(vec![
        na::Point3::new(1.0, 2.0, 3.0),
        na::Point3::new(4.0, 5.0, 6.0),
    ]);

    let json = serde_json::to_string(&src).unwrap();
    assert_eq!(json, "[[1.0,2.0,3.0],[4.0,5.0,6.0]]");

    let dst: KdTree3<na::Point3<f64>> = serde_json::from_str(&json).unwrap();
    assert_eq!(src, dst);
}

#[cfg(feature = "rayon")]
#[test]
fn test_rayon() {
    let points = {
        let mut gen3d = random3d_generator();
        vec(1000, |_| gen3d())
    };
    let kdtree1 = KdTree::build_by_ordered_float(points.clone());
    let kdtree2 = KdTree::par_build_by_ordered_float(points.clone());
    assert_eq!(kdtree1, kdtree2);
}
