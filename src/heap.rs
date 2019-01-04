use std::cmp::Ordering;
use std::collections::HashMap;

use priority_queue::PriorityQueue;
use serde::Deserialize;

use crate::graph::{NodeId, Weight};

// https://github.com/Project-OSRM/osrm-backend/blob/5.18/include/util/query_heap.hpp#L195

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Query {
    pub node: NodeId,
    pub parent: NodeId,
    pub weight: Weight,
    pub duration: Weight,
}

/// Reverses the order of comparison checks for `T`.
#[derive(Eq, PartialEq)]
struct Reverse<T>(T)
where
    T: Eq + Ord + PartialEq + PartialOrd;

impl<T> Ord for Reverse<T>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Reverse<T>) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl<T> PartialOrd for Reverse<T>
where
    T: Ord + PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Reverse<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct QueryHeap {
    heap: PriorityQueue<usize, Reverse<Weight>>,
    inserted: Vec<Query>,
    index: HashMap<NodeId, usize>,
}

impl QueryHeap {
    pub fn new() -> Self {
        QueryHeap {
            heap: PriorityQueue::new(),
            inserted: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// Inserts the query into the priority queue, updating the existing query
    /// if one exists with the same `node`.
    pub fn push(&mut self, query: Query) {
        match self.index.get(&query.node) {
            Some(idx) => {
                self.heap.push(*idx, Reverse(query.weight));
                self.inserted[*idx] = query;
            }
            None => {
                let idx = self.inserted.len();
                self.heap.push(idx, Reverse(query.weight));
                self.index.insert(query.node, idx);
                self.inserted.push(query);
            }
        }
    }

    /// Pops the query with the smallest weight from the priority queue.
    pub fn pop(&mut self) -> Option<Query> {
        self.heap.pop().map(|(idx, _)| self.inserted[idx])
    }

    /// Gets the query involving the given node, if we've seen one before.
    /// Returns None if the node has never been seen.
    pub fn get(&self, node: NodeId) -> Option<Query> {
        self.index.get(&node).map(|idx| self.inserted[*idx])
    }
}
