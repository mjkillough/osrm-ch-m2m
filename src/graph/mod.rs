mod storage;

use std::ops::Not;
use std::path::Path;

use bitvec::BitVec;

use self::storage::{EdgeArrayEntry, NodeArrayEntry};
use crate::errors::*;

pub use self::storage::{EdgeWeight, NodeId};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

impl Not for Direction {
    type Output = Direction;

    fn not(self) -> Self::Output {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }
}

#[derive(Debug)]
pub struct Edge {
    pub target: NodeId,
    pub weight: EdgeWeight,
    pub duration: EdgeWeight,

    // TODO: Are these mutually exclusive? If so, we can treat as an enum.
    // OSRM stores them as two bools, which allows them to equal each other.
    pub forward: bool,
    pub backward: bool,
}

impl From<&EdgeArrayEntry> for Edge {
    fn from(entry: &EdgeArrayEntry) -> Edge {
        Edge {
            target: entry.target,
            weight: entry.weight,
            duration: entry.duration(),
            forward: entry.forward(),
            backward: entry.backward(),
        }
    }
}

pub struct Graph {
    nodes: Vec<NodeArrayEntry>,
    edges: Vec<EdgeArrayEntry>,
    include_edges: BitVec,
}

impl Graph {
    pub fn from_file(tar: impl AsRef<Path>) -> Result<Self> {
        Ok(Graph {
            nodes: storage::read_array(tar.as_ref(), storage::NODE_ARRAY_PATH)?,
            edges: storage::read_array(tar.as_ref(), storage::EDGE_ARRAY_PATH)?,
            include_edges: storage::read_bit_array(tar.as_ref(), storage::EDGE_FILTER_PATH)?,
        })
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_adjacent_edges(
        &self,
        node_id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge> + '_ {
        let node_id = node_id as usize;
        let node = &self.nodes[node_id];
        // TODO: Figure out why this never panics?
        let next_node = &self.nodes[node_id + 1];

        let edges_start = node.first_edge as usize;
        let edges_end = next_node.first_edge as usize;
        let v = self.edges[edges_start..edges_end]
            .iter()
            .enumerate()
            .filter(move |(i, _)| self.include_edge(edges_start + *i))
            .filter(move |(_, entry)| {
                (direction == Direction::Forward && entry.forward())
                    || (direction == Direction::Backward && entry.backward())
            })
            .map(|(_, entry)| entry.into())
            .collect::<Vec<_>>();
        v.into_iter()
    }

    fn include_edge(&self, edge_id: usize) -> bool {
        self.include_edges.get(edge_id)
    }
}
