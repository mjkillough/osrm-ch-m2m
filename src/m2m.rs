use std::collections::HashMap;

use itertools::Itertools;
use rayon::prelude::*;
use superslice::Ext;

use crate::graph::{
    Direction::{self, *},
    Graph, NodeId, Weight,
};
use crate::heap::{Query, QueryHeap};

// NodeBucket:
// https://github.com/Project-OSRM/osrm-backend/blob/5.18/include/engine/routing_algorithms/many_to_many.hpp
#[derive(Clone)]
struct Bucket {
    middle_node: NodeId,
    column_index: usize,
    weight: Weight,
    duration: Weight,
}

// Rest of this file:
// https://github.com/Project-OSRM/osrm-backend/blob/5.18/src/engine/routing_algorithms/many_to_many_ch.cpp

struct Search<'a> {
    graph: &'a Graph,
    heap: QueryHeap,
    direction: Direction,
    index: usize,
}

impl<'a> Search<'a> {
    fn new(graph: &'a Graph, direction: Direction, queries: Vec<Query>, index: usize) -> Self {
        let mut heap = QueryHeap::new();
        for query in queries {
            heap.push(query);
        }
        Search {
            graph,
            heap,
            direction,
            index,
        }
    }

    fn perform(mut self) -> Vec<Bucket> {
        let mut buckets = Vec::new();

        while let Some(query) = self.heap.pop() {
            buckets.push(Bucket {
                middle_node: query.node,
                column_index: self.index,
                weight: query.weight,
                duration: query.duration,
            });

            self.relax_outgoing_edges(self.direction, query.node, query.weight, query.duration);
        }

        buckets
    }

    fn stall_at_node(&self, direction: Direction, node: NodeId, weight: Weight) -> bool {
        for edge in self.graph.get_adjacent_edges(node, !direction) {
            if let Some(query) = self.heap.get(edge.target) {
                if query.weight + edge.weight < weight {
                    return true;
                }
            }
        }
        false
    }

    fn relax_outgoing_edges(
        &mut self,
        direction: Direction,
        node: NodeId,
        weight: Weight,
        duration: Weight,
    ) {
        if self.stall_at_node(direction, node, weight) {
            return;
        }

        for edge in self.graph.get_adjacent_edges(node, direction) {
            let query = Query {
                node: edge.target,
                parent: node,
                weight: weight + edge.weight,
                duration: duration + edge.duration(),
            };

            if let Some(current) = self.heap.get(edge.target) {
                if (query.weight, query.duration) < (current.weight, current.duration) {
                    self.heap.push(query)
                }
            } else {
                self.heap.push(query)
            }
        }
    }
}

/// Combines the buckets from the forward/backwards searches, to produce shortest paths.
struct BucketJoiner<'a> {
    graph: &'a Graph,
    results: &'a mut [Option<(Weight, Weight)>],
    target_buckets: &'a [Bucket],
    source_buckets: &'a [Bucket],
}

impl<'a> BucketJoiner<'a> {
    fn new(
        graph: &'a Graph,
        results: &'a mut [Option<(Weight, Weight)>],
        target_buckets: &'a [Bucket],
        source_buckets: &'a [Bucket],
    ) -> Self {
        BucketJoiner {
            graph,
            results,
            target_buckets,
            source_buckets,
        }
    }

    fn perform(self) {
        for source in self.source_buckets {
            let range = self
                .target_buckets
                .equal_range_by_key(&source.middle_node, |bucket| bucket.middle_node);

            for target in &self.target_buckets[range] {
                let idx = target.column_index;
                let current = self.results[idx];

                let new_weight = source.weight + target.weight;
                let new_duration = source.duration + target.duration;
                let new = (new_weight, new_duration);

                if new_weight < 0 {
                    if let Some((loop_weight, loop_duration)) =
                        self.should_add_loop_weight(source.middle_node, new_weight, new_duration)
                    {
                        if Some((loop_weight, loop_duration)) < current {
                            self.results[idx] = Some((loop_weight, loop_duration));
                        }
                    }
                } else if current.is_none() {
                    self.results[idx] = Some(new);
                } else if let Some(current) = current {
                    if new < current {
                        self.results[idx] = Some(new);
                    }
                }
            }
        }
    }

    fn should_add_loop_weight(
        &self,
        node: NodeId,
        weight: Weight,
        duration: Weight,
    ) -> Option<(Weight, Weight)> {
        // Special case for CH when contractor creates a loop edge node->node.
        assert!(weight < 0);

        if let Some(loop_weight) = self.get_loop_weight(node, false) {
            let new_weight_with_loop = weight + loop_weight;
            if new_weight_with_loop >= 0 {
                let loop_duration = self.get_loop_weight(node, true).unwrap();
                let new_duration_with_loop = duration + loop_duration;
                return Some((new_weight_with_loop, new_duration_with_loop));
            }
        }

        // No loop found or adjusted weight is negative.
        None
    }

    fn get_loop_weight(&self, node: NodeId, use_duration: bool) -> Option<Weight> {
        let mut loop_weight = None;
        for edge in self.graph.get_adjacent_edges(node, Forward) {
            if node == edge.target {
                let value = if use_duration {
                    edge.duration()
                } else {
                    edge.weight
                };
                loop_weight = loop_weight
                    .map(|weight| Weight::min(weight, value))
                    .or_else(|| Some(value));
            }
        }
        loop_weight
    }
}

fn compute_buckets(
    graph: &Graph,
    direction: Direction,
    queries: &mut Vec<(usize, Vec<Query>)>,
) -> Vec<(usize, Vec<Bucket>)> {
    queries
        .split_off(0)
        .into_par_iter()
        .map(|(id, queries)| {
            let buckets = Search::new(graph, direction, queries, id).perform();
            (id, buckets)
        })
        .collect()
}

/// Performs a many-to-many search between source/target nodes.
///
/// It uses rayon to parellise the search. It keeps the intermediate results
/// in memory, so that the matrix can be more efficiently recomputed when the
/// source/target nodes change.
pub struct ManyToMany<'a> {
    graph: &'a Graph,

    num_targets: usize,

    // These are essentially PhantomNodes from the OSRM implementation.
    // The nearest node from the CH graph is found for each input co-ordinate, and then
    // 1 or 2 (depending on forward/backwards edges from the node) queries are created from it.
    source_queries: Vec<(usize, Vec<Query>)>,
    target_queries: Vec<(usize, Vec<Query>)>,

    source_buckets: HashMap<usize, Vec<Bucket>>,
    target_buckets: Vec<Bucket>,

    results: Vec<Vec<Option<(Weight, Weight)>>>,
}

impl<'a> ManyToMany<'a> {
    pub fn new(
        graph: &'a Graph,
        source_queries: Vec<(usize, Vec<Query>)>,
        target_queries: Vec<(usize, Vec<Query>)>,
    ) -> Self {
        let num_targets = target_queries.len();
        ManyToMany {
            graph,
            num_targets,
            source_queries,
            target_queries,
            source_buckets: HashMap::new(),
            target_buckets: Vec::new(),
            results: Vec::new(),
        }
    }

    pub fn add_source(&mut self, source: (usize, Vec<Query>)) {
        self.source_queries.push(source);
    }

    pub fn add_target(&mut self, target: (usize, Vec<Query>)) {
        self.target_queries.push(target);
        self.num_targets += 1;
    }

    fn compute_new_targets(&mut self) {
        let num_new_targets = self.target_queries.len();
        let target_buckets = compute_buckets(&self.graph, Backward, &mut self.target_queries);

        let mut target_buckets = target_buckets
            .into_iter()
            .map(|(_, buckets)| buckets)
            .flatten()
            .collect::<Vec<_>>();
        target_buckets.par_sort_unstable_by_key(|bucket| bucket.middle_node);

        let graph = &self.graph;
        let source_buckets = &self.source_buckets;

        // Compute the new values each row. We pass all the source buckets,
        // but as we only pass the new target buckets, it won't recompute the
        // existing results.
        self.results
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, row)| {
                // TODO: Consider making the entire results matrix contiguous in memory,
                // and resizing all rows/columns in one set of operations.
                row.resize(row.len() + num_new_targets, None);

                let source_buckets = source_buckets.get(&idx).unwrap();
                BucketJoiner::new(graph, row, &target_buckets, source_buckets).perform();
            });

        // Merge two sorted vec of buckets:
        self.target_buckets = std::mem::replace(&mut self.target_buckets, Vec::new())
            .into_iter()
            .merge_by(target_buckets, |a, b| a.middle_node < b.middle_node)
            .collect();
    }

    fn compute_new_sources(&mut self) {
        let num_new_sources = self.source_queries.len();
        let source_buckets = compute_buckets(&self.graph, Forward, &mut self.source_queries);

        let iter = source_buckets
            .into_par_iter()
            .map(|(idx, source_buckets)| {
                let mut results = vec![None; self.num_targets];

                BucketJoiner::new(
                    &self.graph,
                    &mut results,
                    &self.target_buckets,
                    &source_buckets,
                )
                .perform();

                (idx, source_buckets, results)
            })
            .collect::<Vec<_>>();

        self.results
            .resize_with(self.results.len() + num_new_sources, Vec::new);

        for (idx, source_buckets, results) in iter {
            self.results[idx] = results;
            self.source_buckets.insert(idx, source_buckets);
        }
    }

    pub fn compute(&mut self) -> &Vec<Vec<Option<(Weight, Weight)>>> {
        if !self.target_queries.is_empty() {
            self.compute_new_targets();
        }
        if !self.source_queries.is_empty() {
            self.compute_new_sources();
        }

        &self.results
    }
}

/// Performs a non-parallel many-to-many search.
///
/// Does not support any incremental searches. This exists to demonstrate the
/// the top-level structure of the algorithm, when we aren't worried about
/// caching intermediate results.
pub fn many_to_many(
    graph: &Graph,
    source_queries: Vec<Vec<Query>>,
    target_queries: Vec<Vec<Query>>,
) -> Vec<Vec<Option<(Weight, Weight)>>> {
    let num_targets = target_queries.len();
    let mut target_buckets = target_queries
        .into_iter()
        .enumerate()
        .map(|(idx, queries)| Search::new(graph, Backward, queries, idx).perform())
        .flatten()
        .collect::<Vec<_>>();

    target_buckets.sort_by_key(|bucket| bucket.middle_node);

    source_queries
        .into_iter()
        .enumerate()
        .map(|(idx, queries)| Search::new(graph, Forward, queries, idx).perform())
        .map(|source_buckets| {
            let mut results = vec![None; num_targets];
            BucketJoiner::new(graph, &mut results, &target_buckets, &source_buckets).perform();
            results
        })
        .collect()
}
