use petgraph::graph::NodeIndex;
// Structures used
//use counter::Counter;
//use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::collections::HashMap;
use twox_hash::{xxhash64, XxHash64};

// Petgraph types
use petgraph::EdgeType;

// Reading a graph from a txt file
use std::fs::File;

// Writing the graph to a dotfile
use palette::{Hsv, IntoColor, Srgb};
use petgraph::dot::{Config, Dot};
use std::collections::HashSet;
use std::fmt::Debug;
use std::io::Write;

use petgraph::visit::GraphProp;
use petgraph::Directed;
use petgraph::Direction::{Incoming, Outgoing};

// Two methods for defining a graph type that we are opterating on

// Runtime check to see if a graph is directed. Simpler but less idiomatic
fn is_directed<G>(_graph: &G) -> bool
where
    G: GraphProp,
{
    std::any::type_name::<G::EdgeType>() == std::any::type_name::<Directed>()
}

// A custom trait for the WL dimension. This is a bit more complex, but limits the if/else clutter and runtime checks in the code
pub trait WLdim {}
pub struct OneWL;
pub struct TwoWL;
impl WLdim for OneWL {}
impl WLdim for TwoWL {}

// Struct that holds the necessary fields and methods to run WL
pub struct GraphWrapper<N, E, Ty, Wd>
where
    N: std::cmp::Ord, // Nodeweight
    Ty: EdgeType,     // Directed or undirected
    Wd: WLdim,
{
    pub graph: Graph<N, E, Ty>,
    seed: u64,
    labels: Vec<u64>,
    new_labels: Vec<u64>, // To store newly calculated labels (cannot be done in place)
    niters: usize,        // After how many iterations to terminate
    check_stable: bool,   // Whether to terminate once the colouring becomes stable
    get_subgraphs: bool,  // Whether to store the subgraph hashes
    pub subgraphs: Option<Vec<Vec<u64>>>, // In case we're doing subgraph hashing
    _dim: std::marker::PhantomData<Wd>, // Marker for the WL dimension
}

// Implementations specifically for 1-dimensional WL
impl<N, E, Ty> GraphWrapper<N, E, Ty, OneWL>
where
    N: std::cmp::Ord,
    Ty: EdgeType,
{
    // Make a new wrapper based on the input graph
    pub fn new(
        graph: Graph<N, E, Ty>,
        seed: u64,
        mut niters: usize,
        check_stable: bool,
        sub: bool,
    ) -> Self {
        let labels = Vec::with_capacity(graph.node_count());
        let new_labels = vec![0; graph.node_count()]; // interesting: capacity vs length!
        if niters == 0 || niters > graph.node_count() {
            niters = graph.node_count() - 1;
        }

        // allocate the vector of vectors to store neighbourhoods hashes, if necessary
        let subgraphs = if sub {
            Some(vec![Vec::with_capacity(niters); graph.node_count()])
        } else {
            None
        };
        GraphWrapper {
            graph,
            seed,
            labels,
            new_labels,
            niters,
            check_stable,
            get_subgraphs: sub,
            subgraphs,
            _dim: std::marker::PhantomData,
        }
    }

    // Run 1-dimensional WL on the graph
    pub fn run(&mut self) {
        self.initial_graph();
        let mut its = 1;
        while self.check_stable || its < self.niters {
            self.calculate_new_labels();
            its += 1;
            if self.check_stable && self.stabilised() {
                break;
            }
            self.update_graph();
        }
    }

    // Get the labels for the next iteration based on the current state
    fn calculate_new_labels(&mut self) {
        for node in self.graph.node_indices() {
            // Collect all the relevant hashes: of the node itself and all its neighbours
            let mut input_hashes = Vec::new();
            if !is_directed(&self.graph) {
                for neighbour in self.graph.neighbors(node) {
                    input_hashes.push(self.labels[neighbour.index()]);
                }
                input_hashes.sort_unstable(); // sort for consistency
            } else {
                for neighbour in self.graph.neighbors_directed(node, Incoming) {
                    input_hashes.push(self.labels[neighbour.index()]);
                }
                let mut outgoing_hashes = Vec::new();
                for neighbour in self.graph.neighbors_directed(node, Outgoing) {
                    outgoing_hashes.push(self.labels[neighbour.index()]);
                }

                outgoing_hashes.sort_unstable();

                //separately label the in and outgoing hashes  (Previously had a concern: what if one combination of nodes followed by another and then the node's hash itself also possible in a different way? Seems unlikely -> different hash iteration)
                input_hashes = vec![
                    XxHash64::oneshot(self.seed, bytemuck::cast_slice(&input_hashes)),
                    XxHash64::oneshot(self.seed, bytemuck::cast_slice(&outgoing_hashes)),
                ];
            }

            input_hashes.push(self.labels[node.index()]); // In this way, the hash of the node itself is always the last one of the list!
            let hash = XxHash64::oneshot(self.seed, bytemuck::cast_slice(&input_hashes));
            self.new_labels[node.index()] = hash;
        }
    }

    fn initial_graph(&mut self) {
        // Initial weights are (hashed) degrees Is hashing here even really necessary at all?
        let mut hash: u64;
        if !is_directed(&self.graph) {
            // do this kind of stuff with macros? Is that worth the complexity? Might be good bc repetetive use? Maybe better to just not check at runtime at all..
            for node in self.graph.node_indices() {
                hash = self.graph.neighbors(node).count() as u64;
                self.labels.push(hash);
            }
        } else {
            for node in self.graph.node_indices() {
                let out = self.graph.neighbors_directed(node, Outgoing).count();
                let ing = self.graph.neighbors_directed(node, Incoming).count();
                hash = XxHash64::oneshot(self.seed, bytemuck::cast_slice(&[out, ing]));
                self.labels.push(hash);
            }
        }
        if self.get_subgraphs {
            for node in self.graph.node_indices() {
                self.subgraphs.as_mut().unwrap()[node.index()].push(self.labels[node.index()]);
            }
        }
    }
}

// Implementations specifically for writing it to dotfile, this requires debug.
impl<N, E, Ty> GraphWrapper<N, E, Ty, OneWL>
where
    N: std::cmp::Ord,
    E: Debug,
    Ty: EdgeType,
{
    // Write the final graph to a dot file, with colouring of the nodes based on what colour class they are in
    pub fn write_dot(&self, path: &str) {
        let hash_to_colour = self.get_colour_map();

        // get a new graph with the colour strings as weights
        let graph = self.graph.map(
            |index, _weight| hash_to_colour[&self.labels[index.index()]].clone(), // Get the colour that belongs to the hash
            |_index, weight| weight, // For edges, simply return the input weight
        );

        // Create a file, create a Dot formatter from petgraph and write that to the file
        let mut f = File::create(path).expect("failed to create the dot file");
        let dot = Dot::with_attr_getters(
            &graph,
            &[Config::NodeIndexLabel, Config::EdgeNoLabel],
            &|_graph, _edge| String::new(),
            &|_graph, node| node.1.to_string(),
        );
        f.write_all(format!("{:?}", dot).as_bytes())
            .expect("failed to write from input to file");
    }

    // Get a hashmap that translates labels (hashes) to associated colours:
    // find the unique labels, get the same number of contrasting colours and finally zip that into a hashmap
    fn get_colour_map(&self) -> HashMap<&u64, String> {
        let unique_hashes: Vec<_> = HashSet::<_>::from_iter(self.labels.iter())
            .into_iter()
            .collect();

        let hash_to_colour = if unique_hashes.len() > 8 {
            // Map hashes to numbers
            unique_hashes
                .iter()
                .enumerate()
                .map(|(i, &hash)| (hash, format!("label = {}", i)))
                .collect()
        } else {
            // Map hashes to contrasting colors
            let colours = generate_contrasting_colors(unique_hashes.len()).map(|c| {
                format!(
                    "style = filled fillcolor= \"#{:02X}{:02X}{:02X}\"",
                    c.red, c.green, c.blue
                )
            });

            unique_hashes.iter().copied().zip(colours).collect()
        };

        hash_to_colour
    }
}

// Get colours that are as opposing as possible
fn generate_contrasting_colors(n: usize) -> impl Iterator<Item = Srgb<u8>> {
    (0..n).map(move |i| {
        let contrast = (360.0 / n as f32) * i as f32; // Spread hues (for colours) and lightness (for black and white) evenly lightness doesn't do what was hoped :(
        let hsv = Hsv::new(contrast, 1.0, 1.0); // Full saturation
        let srgb: Srgb = hsv.into_color();
        srgb.into_format() // Convert to u8 format
    })
}

// Implementations specifically for 2-dimensional WL
impl<N, E, Ty> GraphWrapper<N, E, Ty, TwoWL>
where
    N: std::cmp::Ord,
    Ty: EdgeType,
{
    // Make a new wrapper based on the input graph
    pub fn new_2wl(
        graph: Graph<N, E, Ty>,
        seed: u64,
        mut niters: usize,
        check_stable: bool,
        sub: bool,
    ) -> Self {
        if sub {
            panic!("Subgraph hashing is not supported for 2-dimensional WL");
        }
        if is_directed(&graph) {
            panic!("Directed graphs are not yet supported for 2-dimensional WL");
        }
        let number_tuples = ((graph.node_count() - 1)
            .checked_pow(2)
            .expect("This grapsize exceeds support for 2-dimensional WL")
            + graph.node_count()
            - 1)
            / 2
            + graph.node_count();
        let labels = Vec::with_capacity(number_tuples);
        let new_labels = vec![0; number_tuples];
        if niters == 0 || niters > number_tuples {
            niters = number_tuples - 1;
        }

        let subgraphs = None;
        GraphWrapper {
            graph,
            seed,
            labels,
            new_labels,
            niters,
            check_stable,
            get_subgraphs: sub,
            subgraphs,
            _dim: std::marker::PhantomData,
        }
    }

    // Run 2-dimensional WL on the graph.
    // Unfortunately a duplicate of the code for 1-dimensional WL. This was necessary because otherwise there is difficulty with scoping of the methods.
    pub fn run(&mut self) {
        self.initial_graph();
        let mut its = 1;
        while self.check_stable || its < self.niters {
            self.calculate_new_labels();
            its += 1;
            if self.check_stable && self.stabilised() {
                break;
            }
            self.update_graph();
        }
    }

    fn initial_graph(&mut self) {
        for left in 0..self.graph.node_count() {
            let left_node = NodeIndex::new(left);
            for right in 0..=left {
                self.labels.push(
                    self.graph
                        .edges_connecting(left_node, NodeIndex::new(right))
                        .count() as u64,
                )
            }
        }
    }

    // Get the labels for the next iteration based on the current state
    fn calculate_new_labels(&mut self) {
        for left in 0..self.graph.node_count() {
            for right in 0..=left {
                let mut input_hashes: Vec<[u64; 2]> = Vec::with_capacity(self.graph.node_count());
                for alternative in 0..self.graph.node_count() {
                    let left_replace = self.labels[get_label_index(alternative, right)]; // Better way to access?
                    let right_replace = self.labels[get_label_index(left, alternative)];
                    if left_replace < right_replace {
                        input_hashes.push([left_replace, right_replace]);
                    } else {
                        input_hashes.push([right_replace, left_replace])
                    }
                }
                input_hashes.sort_unstable();
                // Technically faster to allocate first, though I don't really see that in the results yet.
                let mut flat: Vec<u64> = Vec::with_capacity(input_hashes.len() * 2 + 1);
                flat.extend(input_hashes.into_iter().flatten());
                let current_index = get_label_index(left, right);
                flat.push(self.labels[current_index]);
                let hash = XxHash64::oneshot(self.seed, bytemuck::cast_slice(&flat));
                self.new_labels[current_index] = hash;
            }
        }
    }
}

fn get_label_index(mut left: usize, mut right: usize) -> usize {
    if right > left {
        (left, right) = (right, left);
    }
    (left * left + left) / 2 + right
}

// Implementations generic for all WL dimensions
impl<N, E, Ty, Wd> GraphWrapper<N, E, Ty, Wd>
where
    N: std::cmp::Ord,
    Ty: EdgeType,
    Wd: WLdim,
{
    // Maps labels from the previous round to their new values. Iff all labels that were the same are still the same colouring has stabilised
    fn stabilised(&self) -> bool {
        let mut label_mapping: HashMap<u64, u64, xxhash64::State> =
            HashMap::with_hasher(xxhash64::State::with_seed(self.seed));
        for (idx, old_hash) in self.labels.iter().enumerate() {
            match label_mapping.get(old_hash) {
                Some(new_hash) => {
                    if self.new_labels[idx] != *new_hash {
                        return false;
                    }
                }
                None => {
                    label_mapping.insert(*old_hash, self.new_labels[idx]);
                }
            }
        }
        true
    }

    // Update the labels in the graph with the values calculated in the last round.
    // If we're doing subgraph hashing, store those in the array.
    fn update_graph(&mut self) {
        if self.get_subgraphs {
            for (idx, hash) in self.new_labels.iter().enumerate() {
                self.subgraphs.as_mut().unwrap()[idx].push(*hash);
            }
        }
        std::mem::swap(&mut self.labels, &mut self.new_labels);
    }

    // Get the final graph hash
    pub fn get_results(&mut self) -> u64 {
        self.labels.sort_unstable(); // unstable is faster than 'normal' sort
        XxHash64::oneshot(self.seed, bytemuck::cast_slice(&self.labels))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::{DiGraph, UnGraph};

    #[test]
    fn simplest() {
        let g = UnGraph::<(), ()>::from_edges([(0, 1)]);
        let g2 = UnGraph::<(), ()>::from_edges([(1, 0)]);
        let mut wl1 = GraphWrapper::new(g, 42, 0, true, false);
        let mut wl2 = GraphWrapper::new(g2, 42, 0, true, false);
        wl1.run();
        wl2.run();
        assert_eq!(wl1.get_results(), wl2.get_results());
    }
    #[test]
    fn simple_fail() {
        let g = UnGraph::<(), ()>::from_edges([(0, 1), (1, 2)]);
        let g2 = UnGraph::<(), ()>::from_edges([(1, 0)]);
        let mut wl1 = GraphWrapper::new_2wl(g, 42, 0, true, false);
        let mut wl2 = GraphWrapper::new(g2, 42, 0, true, false);
        wl1.run();
        wl2.run();
        assert_ne!(wl1.get_results(), wl2.get_results());
    }
    #[test]
    fn different_iterations() {
        let g = UnGraph::<(), ()>::from_edges([(0, 1), (0, 2), (0, 3)]);
        let mut wl1 = GraphWrapper::new(g.clone(), 42, 2, false, false);
        let mut wl2 = GraphWrapper::new(g, 42, 3, false, false);
        wl1.run();
        wl2.run();
        assert_ne!(wl1.get_results(), wl2.get_results());
    }
    #[test]
    fn early_termination() {
        let g = UnGraph::<(), ()>::from_edges([(0, 1), (0, 2), (0, 3)]);
        let mut wl1 = GraphWrapper::new(g.clone(), 42, 0, false, false);
        let mut wl2 = GraphWrapper::new(g, 42, 0, true, false);
        wl1.run();
        wl2.run();
        assert_ne!(wl1.get_results(), wl2.get_results()); // these have different outcomes, that is important to be aware of!
    }
    #[test]
    fn equivalence_hardcoded_stabilisation() {
        // Same example as in proposal. NB how confusing this is, a.o. because the autostabilisation skips updating the graph once stabilisation is confirmed
        let g = UnGraph::<(), ()>::from_edges([(0, 1), (1, 2), (2, 3), (3, 4)]);
        let mut wl1 = GraphWrapper::new(g.clone(), 42, 2, false, false);
        let mut wl2 = GraphWrapper::new(g, 42, 0, true, false);
        wl1.run();
        wl2.run();
        assert_eq!(wl1.get_results(), wl2.get_results());
    }
    #[test]
    fn simple_dir() {
        let g = UnGraph::<(), ()>::from_edges([(0, 1)]);
        let g2 = DiGraph::<(), ()>::from_edges([(0, 1)]);
        let mut wl1 = GraphWrapper::new(g, 42, 0, true, false);
        let mut wl2 = GraphWrapper::new(g2, 42, 0, true, false);
        wl1.run();
        wl2.run();
        assert_ne!(wl1.get_results(), wl2.get_results());
    }
    #[test]
    fn flipped_dir() {
        let g = DiGraph::<(), ()>::from_edges([(0, 1), (1, 2), (3, 4), (2, 3)]);
        let g2 = DiGraph::<(), ()>::from_edges([(1, 0), (2, 1), (3, 2), (4, 3)]);
        let mut wl1 = GraphWrapper::new(g, 42, 0, true, false);
        let mut wl2 = GraphWrapper::new(g2, 42, 0, true, false);
        wl1.run();
        wl2.run();
        assert_eq!(wl1.get_results(), wl2.get_results());
    }

    #[test]
    fn flipped_middle() {
        let g = DiGraph::<(), ()>::from_edges([(0, 1), (1, 2), (2, 3), (3, 4)]);
        let g2 = DiGraph::<(), ()>::from_edges([(1, 0), (2, 1), (2, 3), (4, 3)]);
        let mut wl1 = GraphWrapper::new(g, 42, 0, true, false);
        let mut wl2 = GraphWrapper::new(g2, 42, 0, true, false);
        wl1.run();
        wl2.run();
        assert_ne!(wl1.get_results(), wl2.get_results());
    }
    #[test]
    fn flipped_middle_undirected() {
        let g = UnGraph::<(), ()>::from_edges([(0, 1), (1, 2), (2, 3), (3, 4)]);
        let g2 = UnGraph::<(), ()>::from_edges([(1, 0), (2, 1), (2, 3), (4, 3)]);
        let mut wl1 = GraphWrapper::new(g, 42, 0, true, false);
        let mut wl2 = GraphWrapper::new(g2, 42, 0, true, false);
        wl1.run();
        wl2.run();
        assert_eq!(wl1.get_results(), wl2.get_results());
    }

    // #[test]
    // fn examples_practical_isomorphism() {
    //     let g = ungraph_from_edgelist("graphs/practical/is-iso1.edgelist");
    //     let f = ungraph_from_edgelist("graphs/practical/is-iso2.edgelist");
    //     let mut wl_graphg = GraphWrapper::new(g, 42, 0, true);
    //     let mut wl_graphf = GraphWrapper::new(f, 42, 0, true);
    //     // This is the test based on the practical isomorphism paper where two of them are actually isomorphic, so important it does not return difference:
    //     assert_eq!(wl_graphf.run(), wl_graphg.run());

    //     let g = ungraph_from_edgelist("graphs/practical/not-iso1.edgelist");
    //     let f = ungraph_from_edgelist("graphs/practical/not-iso1.edgelist");
    //     let mut wl_graphg = GraphWrapper::new(g, 42, 0, true);
    //     let mut wl_graphf = GraphWrapper::new(f, 42, 0, true);
    //     // Here they are *not* isomorphic, but is constructed in such a way WL cannot distinguish it, so again relevant it would not return unequal:
    //     assert_eq!(wl_graphf.run(), wl_graphg.run());
    // }
    // #[test] # doesn't work on git since i'm not pushing graphs -- probably should reconsider?
    // fn weird_test() {
    //     let g = ungraph_from_edgelist("graphs/rantree-iso/rantree-000020.edgelist");
    //     let f = ungraph_from_edgelist("graphs/rantree-iso/rantree-000020-iso.edgelist");
    //     let mut wl_graphg = GraphWrapper::new(g, 42, 0, true);
    //     let mut wl_graphf = GraphWrapper::new(f, 42, 0, true);
    //     // This is the test based on the practical isomorphism paper where two of them are actually isomorphic, so important it does not return difference:
    //     assert_eq!(wl_graphf.run(), wl_graphg.run());
    // }
}
