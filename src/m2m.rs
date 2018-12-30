use superslice::Ext;

use crate::errors::*;
use crate::graph::{Direction, EdgeWeight, Graph, NodeId};
use crate::heap::{Query, QueryHeap};

struct NodeBucket {
    middle_node: NodeId,
    parent_node: NodeId,
    column_index: usize, // column in weight/duration matrix ???
    weight: EdgeWeight,
    duration: EdgeWeight,
}

pub struct ManyToMany {
    source_queries: Vec<Vec<Query>>,
    target_queries: Vec<Vec<Query>>,
    num_sources: usize,
    num_targets: usize,

    pub results: Vec<Option<(EdgeWeight, EdgeWeight)>>,

    graph: Graph,
    buckets: Vec<NodeBucket>,
    heap: QueryHeap,
}

impl ManyToMany {
    pub fn new() -> Result<ManyToMany> {
        let target_queries = vec![vec![
            Query {
                node: 861677,
                weight: 77,
                parent: 861677,
                duration: 77,
            },
            Query {
                node: 861680,
                weight: 29,
                parent: 861680,
                duration: 29,
            },
        ]];
        let source_queries = vec![vec![
            Query {
                node: 791407,
                weight: -477,
                parent: 791407,
                duration: 477,
            },
            Query {
                node: 791413,
                weight: -87,
                parent: 791413,
                duration: 87,
            },
        ]];
        let num_sources = 1;
        let num_targets = 1;

        let results = vec![None; num_sources * num_targets];

        let buckets = Vec::new();
        let heap = QueryHeap::new();
        let graph = Graph::from_file("data/1.osrm.hsgr")?;

        Ok(ManyToMany {
            source_queries,
            target_queries,
            num_sources,
            num_targets,
            results,
            buckets,
            graph,
            heap,
        })
    }

    pub fn perform(&mut self) {
        for target_idx in 0..self.target_queries.len() {
            for query in &self.target_queries[target_idx] {
                self.heap.push(*query);
            }

            while let Some(query) = self.heap.pop() {
                self.backward_search(query, target_idx);
            }
        }

        self.buckets.sort_by_key(|bucket| bucket.middle_node);

        self.heap = QueryHeap::new();
        for source_idx in 0..self.source_queries.len() {
            for query in &self.source_queries[source_idx] {
                self.heap.push(*query);
            }

            while let Some(query) = self.heap.pop() {
                self.forward_search(query, source_idx);
            }
        }
    }

    fn backward_search(&mut self, query: Query, column: usize) {
        self.buckets.push(NodeBucket {
            middle_node: query.node,
            parent_node: query.parent,
            column_index: column,
            weight: query.weight,
            duration: query.duration,
        });

        self.relax_outgoing_edges(
            Direction::Backward,
            query.node,
            query.weight,
            query.duration,
        );
    }

    fn forward_search(&mut self, query: Query, row: usize) {
        let source_weight = query.weight;
        let source_duration = query.duration;

        let range = self
            .buckets
            .equal_range_by_key(&query.node, |bucket| bucket.middle_node);
        for bucket in &self.buckets[range] {
            let target_weight = bucket.weight;
            let target_duration = bucket.duration;

            let idx = row * self.num_targets + bucket.column_index;
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

        self.relax_outgoing_edges(Direction::Forward, query.node, query.weight, query.duration);
    }

    fn stall_at_node(&self, direction: Direction, node: NodeId, weight: EdgeWeight) -> bool {
        for edge in self.graph.get_adjacent_edges(node, !direction) {
            if let Some(query) = self.heap.get(edge.target) {
                if query.weight + edge.weight < weight {
                    return true;
                }
            }
        }
        return false;
    }

    fn relax_outgoing_edges(
        &mut self,
        direction: Direction,
        node: NodeId,
        weight: EdgeWeight,
        duration: EdgeWeight,
    ) {
        if self.stall_at_node(direction, node, weight) {
            return;
        }

        for edge in self.graph.get_adjacent_edges(node, direction) {
            let query = Query {
                node: edge.target,
                parent: node,
                weight: weight + edge.weight,
                duration: weight + edge.duration,
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

    fn get_loop_weight(&self, node: NodeId, use_duration: bool) -> Option<EdgeWeight> {
        let mut loop_weight = None;
        for edge in self.graph.get_adjacent_edges(node, Direction::Forward) {
            if node == edge.target {
                let value = if use_duration {
                    edge.duration
                } else {
                    edge.weight
                };
                loop_weight = loop_weight
                    .map(|weight| EdgeWeight::min(weight, value))
                    .or(Some(value));
            }
        }
        loop_weight
    }

    fn should_add_loop_weight(
        &self,
        node: NodeId,
        weight: EdgeWeight,
        duration: EdgeWeight,
    ) -> Option<(EdgeWeight, EdgeWeight)> {
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
        return None;
    }
}
