mod errors;
mod graph;
mod heap;

use superslice::Ext;

pub use self::errors::*;
pub use self::graph::Graph;

use self::graph::{Direction, EdgeWeight, NodeId};
use self::heap::{Query, QueryHeap};

struct NodeBucket {
    middle_node: NodeId,
    parent_node: NodeId,
    column_index: usize, // column in weight/duration matrix ???
    weight: EdgeWeight,
    duration: EdgeWeight,
}

fn get_loop_weight(graph: &Graph, node: NodeId, use_duration: bool) -> Option<EdgeWeight> {
    let mut loop_weight = None;
    for edge in graph.get_adjacent_edges(node, Direction::Forward) {
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
    graph: &Graph,
    node: NodeId,
    weight: EdgeWeight,
    duration: EdgeWeight,
) -> Option<(EdgeWeight, EdgeWeight)> {
    // Special case for CH when contractor creates a loop edge node->node.
    assert!(weight < 0);

    if let Some(loop_weight) = get_loop_weight(graph, node, false) {
        let new_weight_with_loop = weight + loop_weight;
        if new_weight_with_loop >= 0 {
            let loop_duration = get_loop_weight(graph, node, true).unwrap();
            let new_duration_with_loop = duration + loop_duration;
            return Some((new_weight_with_loop, new_duration_with_loop));
        }
    }

    // No loop found or adjusted weight is negative.
    return None;
}

fn stall_at_node(
    graph: &Graph,
    heap: &mut QueryHeap,
    direction: Direction,
    node: NodeId,
    weight: EdgeWeight,
) -> bool {
    for edge in graph.get_adjacent_edges(node, !direction) {
        if let Some(query) = heap.get(edge.target) {
            if query.weight + edge.weight < weight {
                return true;
            }
        }
    }
    return false;
}

fn relax_outgoing_edges(
    graph: &Graph,
    heap: &mut QueryHeap,
    direction: Direction,
    node: NodeId,
    weight: EdgeWeight,
    duration: EdgeWeight,
) {
    if stall_at_node(graph, heap, direction, node, weight) {
        return;
    }

    for edge in graph.get_adjacent_edges(node, direction) {
        let query = Query {
            node: edge.target,
            parent: node,
            weight: weight + edge.weight,
            duration: weight + edge.duration,
        };

        if let Some(current) = heap.get(edge.target) {
            if (query.weight, query.duration) < (current.weight, current.duration) {
                heap.push(query)
            }
        } else {
            heap.push(query)
        }
    }
}

fn main() -> Result<()> {
    let graph = Graph::from_file("data/1.osrm.hsgr")?;

    let mut heap = QueryHeap::new();
    heap.push(Query {
        node: 861677,
        weight: 77,
        parent: 861677,
        duration: 77,
    });
    heap.push(Query {
        node: 861680,
        weight: 29,
        parent: 861680,
        duration: 29,
    });

    let mut buckets: Vec<NodeBucket> = Vec::new();

    while !heap.is_empty() {
        let query = heap.pop().unwrap();

        buckets.push(NodeBucket {
            middle_node: query.node,
            parent_node: query.parent,
            column_index: 0, // XXX
            weight: query.weight,
            duration: query.duration,
        });

        relax_outgoing_edges(
            &graph,
            &mut heap,
            Direction::Backward,
            query.node,
            query.weight,
            query.duration,
        );
    }

    println!("Number of buckets: {}", buckets.len());

    buckets.sort_by_key(|bucket| bucket.middle_node);

    let number_of_sources = 1;
    let number_of_targets = 1;
    let number_of_entries = number_of_sources * number_of_targets;

    let mut results_table: Vec<Option<(EdgeWeight, EdgeWeight)>> = vec![None; number_of_entries];

    let row_index = 0;
    let column_index = 0;

    let mut heap = QueryHeap::new();
    heap.push(Query {
        node: 791407,
        weight: -477,
        parent: 791407,
        duration: 477,
    });
    heap.push(Query {
        node: 791413,
        weight: -87,
        parent: 791413,
        duration: 87,
    });
    while !heap.is_empty() {
        let query = heap.pop().unwrap();

        let source_weight = query.weight;
        let source_duration = query.duration;

        let range = buckets.equal_range_by_key(&query.node, |bucket| bucket.middle_node);
        for bucket in &buckets[range] {
            let target_weight = bucket.weight;
            let target_duration = bucket.duration;

            let idx = row_index * number_of_entries + column_index;
            let current = results_table[idx];

            let new_weight = source_weight + target_weight;
            let new_duration = source_duration + target_duration;
            let new = (new_weight, new_duration);

            if new_weight < 0 {
                if let Some((loop_weight, loop_duration)) =
                    should_add_loop_weight(&graph, query.node, new_weight, new_duration)
                {
                    if Some((loop_weight, loop_duration)) < current {
                        results_table[idx] = Some((loop_weight, loop_duration));
                    }
                }
            } else if current.is_none() {
                results_table[idx] = Some(new);
            } else if let Some(current) = current {
                if new < current {
                    results_table[idx] = Some(new);
                }
            }
        }

        relax_outgoing_edges(
            &graph,
            &mut heap,
            Direction::Forward,
            query.node,
            query.weight,
            query.duration,
        );
    }

    println!("Number of buckets: {}", buckets.len());
    println!("Results: {:?}", results_table);

    Ok(())
}
