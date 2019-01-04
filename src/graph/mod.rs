mod storage;

use std::ops::Not;
use std::path::Path;

use bitvec::BitVec;

use self::storage::{EdgeArrayEntry, NodeArrayEntry};
use crate::errors::*;

pub use self::storage::{NodeId, Weight};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

impl Not for Direction {
    type Output = Direction;

    #[inline(always)]
    fn not(self) -> Self::Output {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }
}

pub type Edge = EdgeArrayEntry;

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

    // https://github.com/Project-OSRM/osrm-backend/blob/5.18/include/util/static_graph.hpp#L172-L180

    pub fn get_adjacent_edges(
        &self,
        node_id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = &Edge> + '_ {
        let node_id = node_id as usize;
        let node = &self.nodes[node_id];
        // TODO: Figure out why this never panics?
        let next_node = &self.nodes[node_id + 1];

        let edges_start = node.first_edge as usize;
        let edges_end = next_node.first_edge as usize;
        self.edges[edges_start..edges_end]
            .iter()
            .enumerate()
            .filter(move |(i, _)| self.include_edge(edges_start + *i))
            .filter(move |(_, entry)| {
                (direction == Direction::Forward && entry.forward())
                    || (direction == Direction::Backward && entry.backward())
            })
            .map(|(_, entry)| entry)
    }

    #[inline]
    fn include_edge(&self, edge_id: usize) -> bool {
        self.include_edges.get(edge_id)
    }
}
