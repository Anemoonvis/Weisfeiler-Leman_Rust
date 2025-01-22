//! # Weisfeiler-Leman Graph Isomorphism
//!
//! This crate provides an implementation of the Weisfeiler-Leman (WL) graph isomorphism algorithm for [`petgraph`](https://docs.rs/petgraph/latest/petgraph/) graphs.
//! WL is a sound but incomplete isomorphism test, that because of its speed is often used as a subroutine in complete tests and feature extraction for graph kernels. Additionally, it includes an implementation of the two-dimensional version of the algorithm, which offers greater distinguishing power—particularly for regular graphs—at the cost of a significant runtime penalty.
//!
//! # Example
//!
//! The crate's most basic usage is to compare two graphs for isomorphism using the [`invariant`](fn.invariant.html) function.
//! The function will return the same graph hash for isomorphic graphs (hence it is called an "invariant"), and in most cases different hashes for non-isomorphic graphs.
//! ```rust
//! use petgraph::graph::{UnGraph, DiGraph};
//!
//! let g1 = UnGraph::<u64, ()>::from_edges([(0,1), (1,2), (2,0), (2,3)]);
//! let g2 = UnGraph::<u64, ()>::from_edges([(0,1), (1,2), (2,0), (0,3)]);
//! let g3 = UnGraph::<u64, ()>::from_edges([(0,1), (1,2), (2,3), (0,3)]);
//! let g4 = DiGraph::<u64, ()>::from_edges([(0,1), (1,2), (2,0), (2,3)]);
//! let hash1 = wl_isomorphism::invariant(g1);
//! let hash2 = wl_isomorphism::invariant(g2);
//! let hash3 = wl_isomorphism::invariant(g3);
//! let hash4 = wl_isomorphism::invariant(g4);
//! println!("The final hashes (u64) are:");
//! println!("1: {}, 2: {}, 3: {}, and: {}", hash1, hash2, hash3, hash4);
//! // 1: 16339153988175251892, 2: 16339153988175251892, 3: 14961629621624962419, and: 15573326168912649736
//! # assert_eq!(hash1, hash2);
//! # assert_ne!(hash1, hash3);
//! # assert_ne!(hash1, hash4);
//! ```
//! # IMPORTANT
//! * <b> The WL algorithm is not a complete isomorphism test</b>. This means that when the algorithm returns the same hash for two graphs, they are *possibly* isomorphic, but not guaranteed. On certain classes of graphs (such as random graphs) this is almost always a good indicator of isomorphism, but it is for example not trustworthy on regular graphs. It is, however, a *sound* test, meaning that if the algorithm returns different hashes, the graphs are guaranteed to be non-isomorphic.
//! * <b> Hash values depend on the number of iterations</b>. For algorithms with a fixed iteration count, even the same graph will yield different hashes for different iteration counts.
//! * <b> Hash values depend on device endianness</b>. The same graph will produce different hashes on little-endian and big-endian systems. Compare hashes only on the same device or verify results using example graphs.
//!
//! # Features
//! * <b>Isomorphism testing</b>.  
//!     * Calculate a graph's hash to compare it with other graphs' hashes to determine if they are isomorphic.  
//!     * Use [`invariant`](fn.invariant.html), or if you want the algorithm to run for a specific number of iterations, use [`invariant_iters`](fn.invariant_iters.html).
//!     * Alternatively, use the two-dimensional versions of these, [`invariant_2wl`](fn.invariant_wl.html) and [`iter_2wl`](fn.iter_2wl.html), which offer greater distinguishing power—particularly for regular graphs—at the cost of a significant runtime penalty.
//! * <b>Subgraph hashing </b>.  
//!     * Obtain subgraph hashes for each node at each iteration for tasks like feature extraction for graph kernels.
//!     * Use [`neighbourhood_hash`](fn.neighbourhood_hash.html) for a fixed number of iterations  or [`neighbourhood_stable`](fn.neighbourhood_stable.html) to run until stabilisation.
//! * <b>Dot file output</b>.
//!     * Write the graph to a dot file, where the colour class of each node is visualised.
//!     * Use [`invariant_dot`](fn.invariant_dot.html) or [`iter_dot`](fn.iter_dot.html).
//! * <b>Read from NetworkX edgelist file</b>
//!     * Load graphs from text files in the NetworkX edgelist format.
//!     *  Use [`ungraph_from_edgelist`](fn.ungraph_from_edgelist.html) or [`digraph_from_edgelist`](fn.digraph_from_edgelist.html).
//!

mod graphwrapper; // Declare the graphwrapper module.
use graphwrapper::GraphWrapper; // Re-export GraphWrapper if needed.
use graphwrapper::{OneWL, TwoWL};
use petgraph::Undirected;

use petgraph::graph::{DiGraph, UnGraph};
use petgraph::{EdgeType, Graph};
use std::cmp::Ord;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Calculate the graph invariant using 1-dimensional WL. Automatically stabilises. On graph classes like regular graphs, it is better to use [`invariant_2wl`](fn.invariant_2wl.html), which is more expressive but slower.
pub fn invariant<N: Ord, E, Ty: EdgeType>(graph: Graph<N, E, Ty>) -> u64 {
    let mut wrap: GraphWrapper<N, E, Ty, OneWL> = GraphWrapper::new(graph, 42, 0, true, false);
    wrap.run();
    wrap.get_results()
}

/// Calculate the graph invariant using 2-dimensional WL. Automatically stabilises. This is an implementation of '2-FWL'. This is more expressive than 1-dimensional WL, but much slower. Therefore only use this on graph classes where our default [`invariant`](fn.invariant.html) does not work well.
pub fn invariant_2wl<N: Ord, E>(graph: Graph<N, E, Undirected>) -> u64 {
    let mut wrap: GraphWrapper<N, E, Undirected, TwoWL> =
        GraphWrapper::new_2wl(graph, 42, 0, true, false);
    wrap.run();
    wrap.get_results()
}

/// Calculate the graph invariant using 1-dimensional WL. Runs for `n_iters`. Regular graphs tend to need at most 3 iterations for stabilisation, but for example random trees significantly more. We recommend using [`invariant`](fn.invariant.html) for optimal results, if you don't require a specific number of iterations.
pub fn invariant_iters<N: Ord, E, Ty: EdgeType>(
    graph: Graph<N, E, Ty>,
    n_iters: usize,
) -> u64 {
    let mut wrap = GraphWrapper::new(graph, 42, n_iters, false, false);
    wrap.run();
    wrap.get_results()
}

/// Calculate the graph invariant using 2-dimensional WL. Runs for `n_iters`. We recommend using [`invariant_2wl`](fn.invariant_2wl.html) for optimal results if you don't require a specific number of iterations.
pub fn iter_2wl<N: Ord, E, Ty: EdgeType>(graph: Graph<N, E, Ty>, n_iters: usize) -> u64 {
    let mut wrap = GraphWrapper::new_2wl(graph, 42, n_iters, false, false);
    wrap.run();
    wrap.get_results()
}

/// Generate the subgraph hashes per node per iteration. Can, for example, be used for feature extraction for graph kernels. The computed hash values give some information on the i-hop neighbourhood. The first hash, for example, gives some information on the neighbourhood of each node reachable within one hop. 
/// 
/// In this example, we see each has one neighbour:
/// ```rust
/// use ::petgraph::graph::UnGraph;
///
/// let g1 = UnGraph::<u64, ()>::from_edges([(1, 2), (2, 3), (2, 4), (3, 5), (4, 6), (5, 7), (6, 7)]);
/// let g2 = UnGraph::<u64, ()>::from_edges([(1, 3), (2, 3), (1, 6), (1, 5), (4, 6)]);
/// let g1_hashes = wl_isomorphism::neighbourhood_hash(g1.clone(), 4);
/// let g2_hashes = wl_isomorphism::neighbourhood_hash(g2.clone(), 4);
/// println!("{:?}", g1_hashes[1]);
/// // [1, 1442927345519261537, 353516931035902801, 4661792571936206109]
/// println!("{:?}", g2_hashes[5]);
/// // [1, 1442927345519261537, 353516931035902801, 11058330228393982942]
/// # assert_eq!(g1_hashes[1][0], g2_hashes[5][0]);
/// # assert_eq!(g1_hashes[1][1], g2_hashes[5][1]);
/// # assert_eq!(g1_hashes[1][2], g2_hashes[5][2]);
/// # assert_ne!(g1_hashes[1][3], g2_hashes[5][3]);
/// ```
/// In this example, the neighbourhoods of nodes 1 from g1 and 5 from g2 appear isomorphic up to their 3-hop neighbourhoods, but once the fourth hop is considered you can see they are not.
/// (NB: petgraph introduces an unconnected 0th node in this case, because it uses all node labels from 0 to the highest one indicated. Hence the indexing corresponds to the node's number.)
pub fn neighbourhood_hash<E, Ty: EdgeType>(
    graph: Graph<u64, E, Ty>,
    n_iters: usize,
) -> Vec<Vec<u64>> {
    let mut wrap = GraphWrapper::new(graph, 42, n_iters, false, true);
    wrap.run();
    wrap.subgraphs.unwrap()
}

/// Like [`neighbourhood_hash`](fn.neighbourhood_hash.html), but instead calculated until stability is achieved. (Note that we do not return the last calulated hashes, as these do not provide any new information: they are stable with respect to the last ones that áre returned.)
pub fn neighbourhood_stable<N: Ord, E, Ty: EdgeType>(
    graph: Graph<N, E, Ty>,
) -> Vec<Vec<u64>> {
    let mut wrap = GraphWrapper::new(graph, 42, 0, true, true);
    wrap.run();
    wrap.subgraphs.unwrap()
}

/// Like [`invariant`](fn.invariant.html), but it additionally writes the graph with the final colouring in dot format to `path`.
pub fn invariant_dot<N: Ord, E: Debug, Ty: EdgeType>(graph: Graph<N, E, Ty>, path: &str) -> u64 {
    let mut wrap = GraphWrapper::new(graph, 42, 0, true, false);
    wrap.run();
    wrap.write_dot(path);
    wrap.get_results()
}

/// Like [`invariant_iters`](fn.invariant_iters.html), but it additionally writes the graph with the final colouring in dot format to `path`.
pub fn iter_dot<E: Debug, Ty: EdgeType>(
    graph: Graph<u64, E, Ty>,
    n_iters: usize,
    path: &str,
) -> u64 {
    let mut wrap = GraphWrapper::new(graph, 42, n_iters, false, false);
    wrap.run();
    wrap.write_dot(path);
    wrap.get_results()
}

/// Read an undirected graph from a text file, as produced by [`Networkx.write_edgelist`](https://networkx.org/documentation/stable/reference/readwrite/generated/networkx.readwrite.edgelist.write_edgelist.html). Note that this does not support weights and that if the edgelist skips certain indices, petgraph will infer unconnected nodes at said indices.
pub fn ungraph_from_edgelist(path: &str) -> UnGraph<(), ()> {
    UnGraph::<(), ()>::from_edges(read_edges(path))
}

/// Read a directed graph from a text file, as produced by [`Networkx.write_edgelist`](https://networkx.org/documentation/stable/reference/readwrite/generated/networkx.readwrite.edgelist.write_edgelist.html). Note that this does not support weights and that if the edgelist skips certain indices, petgraph will infer an unconnected node at that index.
pub fn digraph_from_edgelist(path: &str) -> DiGraph<(), ()> {
    DiGraph::<(), ()>::from_edges(read_edges(path))
}

// Read edges from a txt file
fn read_edges(path: &str) -> impl Iterator<Item = (u32, u32)> {
    let file = File::open(path).expect("Unable to open file");
    BufReader::new(file).lines().map(|line| {
        let line = line.expect("Unable to read line");
        let nodes: Vec<&str> = line.split_whitespace().collect();
        (
            nodes[0].parse::<u32>().expect("Couldn't parse"),
            nodes[1].parse::<u32>().expect("Couldn't parse"),
        )
    })
}
