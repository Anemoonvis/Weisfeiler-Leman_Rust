use petgraph::graph::UnGraph;

#[test]
fn equal() {
    let g = UnGraph::<u64, ()>::from_edges([(0, 1), (1, 2), (2, 3), (3, 4)]);
    let g2 = UnGraph::<u64, ()>::from_edges([(1, 0), (2, 1), (2, 3), (4, 3)]);
    assert_eq!(wl::invariant(g.clone()), wl::invariant(g2.clone()));
    assert_eq!(
        wl::invariant_iters(g.clone(), 2),
        wl::invariant_iters(g2.clone(), 2)
    );
    let n_hash = wl::neighbourhood_hash(g.clone(), 3);
    let n_hash2 = wl::neighbourhood_hash(g2.clone(), 3);
    let hash_stable = wl::neighbourhood_stable(g.clone());
    let hash_stable2 = wl::neighbourhood_stable(g2.clone());
    assert!(n_hash == n_hash2);
    assert!(hash_stable == hash_stable2);
}

#[test]
fn unequal_iters() {
    let g = UnGraph::<u64, ()>::from_edges([(0, 1), (1, 2), (2, 3), (3, 4)]);
    assert_ne!(wl::invariant(g.clone()), wl::invariant_iters(g.clone(), 5));
    let n_hash = wl::neighbourhood_hash(g.clone(), 1);
    let n_hash2 = wl::neighbourhood_hash(g.clone(), 4);
    let n_hash_stable = wl::neighbourhood_stable(g.clone());
    assert!(n_hash != n_hash2);
    assert!(n_hash != n_hash_stable);
    assert!(n_hash2 != n_hash_stable);
}

#[test]
fn equal_versions() {
    let g = UnGraph::<u64, ()>::from_edges([(0, 1), (1, 2), (2, 3), (3, 4)]);
    assert_eq!(wl::invariant(g.clone()), wl::invariant_iters(g.clone(), 2));
    let n_hash = wl::neighbourhood_hash(g.clone(), 2);
    let n_hash2 = wl::neighbourhood_hash(g.clone(), 2);
    let n_hash_stable = wl::neighbourhood_stable(g.clone());
    assert!(n_hash == n_hash2);
    assert!(n_hash == n_hash_stable);
}

#[test]
#[ignore]
fn write_dot() {
    let g = UnGraph::<u64, ()>::from_edges([(0, 1), (1, 2), (2, 3), (3, 4)]);
    let a = wl::invariant_dot(g.clone(), "outputs/stable_dot");
    let b = wl::iter_dot(g.clone(), 2, "outputs/iters.dot");
    let c = wl::iter_dot(g.clone(), 3, "outputs/iters_longer.dot");
    let canon = wl::invariant(g);
    assert_eq!(a, b);
    assert_ne!(b, c);
    assert_eq!(a, canon);
}
