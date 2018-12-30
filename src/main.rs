mod errors;
mod graph;
mod heap;

pub use self::errors::*;
pub use self::graph::Graph;

use self::heap::{Query, QueryHeap};

use self::graph::{Direction, EdgeWeight, NodeId};

struct NodeBucket {
    middle_node: NodeId,
    parent_node: NodeId,
    column_index: usize, // column in weight/duration matrix ???
    weight: EdgeWeight,
    duration: EdgeWeight,
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
    println!("1 node {} weight {}", node, weight);
    if stall_at_node(graph, heap, direction, node, weight) {
        println!("2");
        return;
    }

    println!("3");

    for edge in graph.get_adjacent_edges(node, direction) {
        println!("4 to {} weight {} + {}", edge.target, weight, edge.weight);

        let query = Query {
            node: edge.target,
            parent: node,
            weight: weight + edge.weight,
            duration: weight + edge.duration,
        };

        if let Some(existing_query) = heap.get(edge.target) {
            if (query.weight, query.duration) < (existing_query.weight, existing_query.duration) {
                println!("update");
                heap.push(query)
            }
        } else {
            println!("insert");
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

    Ok(())
}
