
# Weisfeiler-Leman Graph Isomorphism

This crate provides an implementation of the Weisfeiler-Leman (WL) graph isomorphism algorithm for [`petgraph`](https://docs.rs/petgraph/latest/petgraph/) graphs.  
WL is a sound but incomplete isomorphism test, that because of its speed is often used as a subroutine in complete tests and feature extraction for graph kernels. Additionally, it includes an implementation of the two-dimensional version of the algorithm, which offers greater distinguishing power—particularly for regular graphs—at the cost of a significant runtime penalty.

# Example

The crate's most basic usage is to compare two graphs for isomorphism using the `invariant` function.  
The function will return the same graph hash for isomorphic graphs (hence it is called an "invariant"), and in most cases different hashes for non-isomorphic graphs.


```rust
use petgraph::graph::{UnGraph, DiGraph};

let g1 = UnGraph::<u64, ()>::from_edges([(0,1), (1,2), (2,0), (2,3)]);
let g2 = UnGraph::<u64, ()>::from_edges([(0,1), (1,2), (2,0), (0,3)]);
let g3 = UnGraph::<u64, ()>::from_edges([(0,1), (1,2), (2,3), (0,3)]);
let g4 = DiGraph::<u64, ()>::from_edges([(0,1), (1,2), (2,0), (2,3)]);
let hash1 = wl::invariant(g1);
let hash2 = wl::invariant(g2);
let hash3 = wl::invariant(g3);
let hash4 = wl::invariant(g4);
println!("The final hashes (u64) are:");
println!("1: {}, 2: {}, 3: {}, and: {}", hash1, hash2, hash3, hash4);
// 1: 16339153988175251892, 2: 16339153988175251892, 3: 14961629621624962419, and: 15573326168912649736
```


## IMPORTANT
- **The WL algorithm is not a complete isomorphism test**. This means that when the algorithm returns the same hash for two graphs, they are *possibly* isomorphic, but not guaranteed. On certain classes of graphs (such as random graphs) this is almost always a good indicator of isomorphism, but it is for example not trustworthy on regular graphs. It is, however, a *sound* test, meaning that if the algorithm returns different hashes, the graphs are guaranteed to be non-isomorphic.
- **Hash values depend on the number of iterations**. For algorithms with a fixed iteration count, even the same graph will yield different hashes for different iteration counts.
- **Hash values depend on device endianness**. The same graph will produce different hashes on little-endian and big-endian systems. Compare hashes only on the same device or verify results using example graphs.

# Features
- **Isomorphism testing**.  
    - Calculate a graph's hash to compare it with other graphs' hashes to determine if they are isomorphic.  
    - Use `invariant`, or if you want the algorithm to run for a specific number of iterations, use `invariant_iters`.
    - Alternatively, use the two-dimensional versions of these, `invariant_2wl` and `iter_2wl`, which offer greater distinguishing power—particularly for regular graphs—at the cost of a significant runtime penalty.
- **Subgraph hashing**.  
    - Obtain subgraph hashes for each node at each iteration for tasks like feature extraction for graph kernels.
    - Use `neighbourhood_hash` for a fixed number of iterations or `neighbourhood_stable` to run until stabilisation.
- **Dot file output**.
    - Write the graph to a dot file, where the colour class of each node is visualised.
    - Use `invariant_dot` or `iter_dot`.
- **Read from NetworkX edgelist file**.
    - Load graphs from text files in the NetworkX edgelist format.
    - Use `ungraph_from_edgelist` or `digraph_from_edgelist`.
