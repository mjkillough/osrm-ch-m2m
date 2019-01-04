use rayon::prelude::*;
use superslice::Ext;

use crate::graph::{Direction, Graph, NodeId, Weight};
use crate::heap::{Query, QueryHeap};

// NodeBucket:
// https://github.com/Project-OSRM/osrm-backend/blob/5.18/include/engine/routing_algorithms/many_to_many.hpp
struct Bucket {
    middle_node: NodeId,
    column_index: usize,
    weight: Weight,
    duration: Weight,
}

// Result of this file:
// https://github.com/Project-OSRM/osrm-backend/blob/5.18/src/engine/routing_algorithms/many_to_many_ch.cpp

struct Search<'a> {
    graph: &'a Graph,
    heap: QueryHeap,
}

impl<'a> Search<'a> {
    fn new(graph: &'a Graph, queries: Vec<Query>) -> Self {
        let mut heap = QueryHeap::new();
        for query in queries {
            heap.push(query);
        }
        Search { graph, heap }
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
                duration: duration + edge.duration,
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

struct BackwardSearch<'a> {
    search: Search<'a>,
    column_index: usize,
}

impl<'a> BackwardSearch<'a> {
    fn new(graph: &'a Graph, queries: Vec<Query>, column_index: usize) -> Self {
        let search = Search::new(graph, queries);
        BackwardSearch {
            search,
            column_index,
        }
    }

    fn perform(mut self) -> Vec<Bucket> {
        let mut buckets = Vec::new();

        while let Some(query) = self.search.heap.pop() {
            buckets.push(Bucket {
                middle_node: query.node,
                column_index: self.column_index,
                weight: query.weight,
                duration: query.duration,
            });

            self.search.relax_outgoing_edges(
                Direction::Backward,
                query.node,
                query.weight,
                query.duration,
            );
        }

        buckets
    }
}

struct ForwardSearch<'a> {
    search: Search<'a>,
    buckets: &'a [Bucket],
    results: Vec<Option<(Weight, Weight)>>,
}

impl<'a> ForwardSearch<'a> {
    fn new(
        graph: &'a Graph,
        buckets: &'a [Bucket],
        queries: Vec<Query>,
        num_targets: usize,
    ) -> Self {
        let search = Search::new(graph, queries);
        let results = vec![None; num_targets];
        ForwardSearch {
            search,
            buckets,
            results,
        }
    }

    fn perform(mut self) -> Vec<Option<(Weight, Weight)>> {
        while let Some(query) = self.search.heap.pop() {
            self.process_query(query);
        }
        self.results
    }

    fn process_query(&mut self, query: Query) {
        let source_weight = query.weight;
        let source_duration = query.duration;

        let range = self
            .buckets
            .equal_range_by_key(&query.node, |bucket| bucket.middle_node);

        for bucket in &self.buckets[range] {
            let target_weight = bucket.weight;
            let target_duration = bucket.duration;

            let idx = bucket.column_index;
            let current = self.results[idx];

            let new_weight = source_weight + target_weight;
            let new_duration = source_duration + target_duration;
            let new = (new_weight, new_duration);

            if new_weight < 0 {
                if let Some((loop_weight, loop_duration)) =
                    self.should_add_loop_weight(query.node, new_weight, new_duration)
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

        self.search.relax_outgoing_edges(
            Direction::Forward,
            query.node,
            query.weight,
            query.duration,
        );
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
        for edge in self
            .search
            .graph
            .get_adjacent_edges(node, Direction::Forward)
        {
            if node == edge.target {
                let value = if use_duration {
                    edge.duration
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

pub fn many_to_many(
    graph: &Graph,
    source_queries: Vec<Vec<Query>>,
    target_queries: Vec<Vec<Query>>,
) -> Vec<Vec<Option<(Weight, Weight)>>> {
    let num_targets = target_queries.len();
    let mut buckets = target_queries
        .into_iter()
        .enumerate()
        .map(|(idx, queries)| BackwardSearch::new(graph, queries, idx).perform())
        .flatten()
        .collect::<Vec<_>>();

    buckets.sort_by_key(|bucket| bucket.middle_node);

    source_queries
        .into_iter()
        .map(|queries| ForwardSearch::new(graph, &buckets, queries, num_targets).perform())
        .collect()
}

// Differs only in `.into_iter()` -> `.into_par_iter()`, but we can't easily make the
// function generic over `Iterator`/`ParallelIterator`.
pub fn parallel_many_to_many(
    graph: &Graph,
    source_queries: Vec<Vec<Query>>,
    target_queries: Vec<Vec<Query>>,
) -> Vec<Vec<Option<(Weight, Weight)>>> {
    let num_targets = target_queries.len();
    let mut buckets = target_queries
        .into_par_iter()
        .enumerate()
        .map(|(idx, queries)| BackwardSearch::new(graph, queries, idx).perform())
        .flatten()
        .collect::<Vec<_>>();

    buckets.sort_by_key(|bucket| bucket.middle_node);

    source_queries
        .into_par_iter()
        .map(|queries| ForwardSearch::new(graph, &buckets, queries, num_targets).perform())
        .collect()
}
