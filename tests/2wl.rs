use petgraph::graph::UnGraph;

#[test]
fn simplest() {
    let g = UnGraph::<(), ()>::from_edges([(0, 1)]);
    let g2 = UnGraph::<(), ()>::from_edges([(1, 0)]);
    assert_eq!(wl::invariant_2wl(g), wl::invariant_2wl(g2));
}
#[test]
fn simple_fail() {
    let g = UnGraph::<(), ()>::from_edges([(0, 1), (1, 2)]);
    let g2 = UnGraph::<(), ()>::from_edges([(1, 0)]);
    assert_ne!(wl::invariant_2wl(g), wl::invariant_2wl(g2));
}
#[test]
fn different_iterations() {
    let g = UnGraph::<(), ()>::from_edges([(0, 1), (0, 2), (0, 3)]);
    assert_ne!(wl::iter_2wl(g.clone(), 2), wl::iter_2wl(g, 3));
}
#[test]
fn equivalence_hardcoded_stabilisation() {
    // Interesting that this has a different number of iterations before stabilisation
    let g = UnGraph::<(), ()>::from_edges([(0, 1), (1, 2), (2, 3), (3, 4)]);
    assert_eq!(wl::iter_2wl(g.clone(), 3), wl::invariant_2wl(g));
}

#[test]
fn flipped_middle_undirected() {
    let g = UnGraph::<(), ()>::from_edges([(0, 1), (1, 2), (2, 3), (3, 4)]);
    let g2 = UnGraph::<(), ()>::from_edges([(1, 0), (2, 1), (2, 3), (4, 3)]);
    assert_eq!(wl::invariant_2wl(g), wl::invariant_2wl(g2));
}

#[test]
fn early_termination_2w() {
    let g = UnGraph::<(), ()>::from_edges([(0, 1), (0, 2), (0, 3)]);
    assert_ne!(wl::invariant_2wl(g.clone()), wl::iter_2wl(g, 0));
}

#[test]
fn extra_expressive() {
    let two_cycles =
        UnGraph::<(), ()>::from_edges([(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3)]);
    let big_cycle = UnGraph::<(), ()>::from_edges([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)]);
    assert_eq!(
        wl::invariant(two_cycles.clone()),
        wl::invariant(big_cycle.clone())
    );
    assert_ne!(wl::invariant_2wl(two_cycles), wl::invariant_2wl(big_cycle));
}
