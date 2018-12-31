use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::ops::Range;
use std::path::Path;

use super::storage::{self, *};
use crate::errors::*;
use crate::{Coordinate, FloatCoordinate};

const BRANCHING_FACTOR: usize = 64;
const LEAF_PAGE_SIZE: usize = 4096;
const LEAF_NODE_SIZE: usize = LEAF_PAGE_SIZE / std::mem::size_of::<TreeNode>();

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TreeIndex {
    level: usize,
    offset: usize,
}

struct CandidateSegment {
    fixed_projected_coordinate: Coordinate,
    edge_data: storage::EdgeBasedNodeSegment, // TODO: encapsulate
}

#[derive(Debug, Eq, PartialEq)]
struct QueryCandidate {
    squared_min_dist: u64,
    tree_index: TreeIndex,
    fixed_projected_coordinate: Option<Coordinate>,
    segment_index: Option<usize>,
}

impl Ord for QueryCandidate {
    fn cmp(&self, other: &QueryCandidate) -> Ordering {
        self.squared_min_dist.cmp(&other.squared_min_dist)
    }
}

impl PartialOrd for QueryCandidate {
    fn partial_cmp(&self, other: &QueryCandidate) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl QueryCandidate {
    fn is_segment(&self) -> bool {
        !self.segment_index.is_none()
    }
}

struct RTree {
    // Representation of the in-memory search tree
    tree: Vec<TreeNode>,
    // Holds the start indexes of each level in m_search_tree
    tree_level_starts: Vec<TreeLevelStart>,

    // Reference to the actual lon/lat data we need for doing math
    coordinates: Vec<Coordinate>,

    // TODO: Decide whether it needs to be mmaped?
    objects: Vec<EdgeBasedNodeSegment>,
}

const TREE_NODE_PATH: &str = "/common/rtree/search_data";
const TREE_LEVEL_STARTS_PATH: &str = "/common/rtree/search_tree_level_starts";
const COORDINATES_PATH: &str = "/common/nbn_data/coordinates";

impl RTree {
    fn from_file(
        ram_index: impl AsRef<Path>,
        nbg_nodes: impl AsRef<Path>,
        file_index: impl AsRef<Path>,
    ) -> Result<RTree> {
        let tree = read_array_from_tar(ram_index.as_ref(), TREE_NODE_PATH)?;
        let tree_level_starts = read_array_from_tar(ram_index, TREE_LEVEL_STARTS_PATH)?;
        let coordinates: Vec<storage::Coordinate> =
            read_array_from_tar(nbg_nodes, COORDINATES_PATH)?;
        let coordinates = coordinates.into_iter().map(Coordinate::from).collect();
        let objects = read_array_from_file(file_index)?;
        Ok(RTree {
            tree,
            tree_level_starts,
            coordinates,
            objects,
        })
    }

    fn nearest<F, T>(
        &self,
        input_coordinate: Coordinate,
        filter: F,
        terminate: T,
    ) -> Vec<storage::EdgeBasedNodeSegment>
    where
        F: Fn(&CandidateSegment) -> (bool, bool),
        T: Fn(usize, &CandidateSegment) -> bool,
    {
        let mut results: Vec<_> = Vec::new();
        let projected_coordinate = FloatCoordinate::from(input_coordinate).from_wgs84();
        let fixed_projected_coordinate = projected_coordinate.into();

        // traversal_queue is a max heap. We don't need to change priorities
        // so we can go with stdlib heap.
        let mut traversal_queue = BinaryHeap::new();
        // Insert root node.
        traversal_queue.push(QueryCandidate {
            squared_min_dist: 0,
            tree_index: TreeIndex {
                level: 0,
                offset: 0,
            },
            fixed_projected_coordinate: None,
            segment_index: None,
        });

        while let Some(query) = traversal_queue.pop() {
            if !query.is_segment() {
                if self.is_leaf(&query.tree_index) {
                    self.explore_leaf_node(
                        &mut traversal_queue,
                        &query.tree_index,
                        &fixed_projected_coordinate,
                        &projected_coordinate,
                    );
                } else {
                    self.explore_tree_node(
                        &mut traversal_queue,
                        &query.tree_index,
                        &fixed_projected_coordinate,
                    );
                }
            } else {
                // The current segment is an actual road.
                let segment_index = query
                    .segment_index
                    .expect("QueryCandidate should have segment_index");
                let fixed_projected_coordinate = query
                    .fixed_projected_coordinate
                    .expect("QueryCandidate should have fixed_projected_coordinate");

                let mut edge_data = self.objects[segment_index].clone();
                let current_candidate = CandidateSegment {
                    fixed_projected_coordinate,
                    edge_data,
                };

                // To allow returns of no-results if too restrictive filtering, this needs to be
                // done here, even though performance would indicate that we want to stop after
                // adding the first candidate.
                if terminate(results.len(), &current_candidate) {
                    break;
                }

                let use_segment = filter(&current_candidate);
                if !use_segment.0 && !use_segment.1 {
                    continue;
                }
                edge_data
                    .forward_segment_id
                    .set_enabled(edge_data.forward_segment_id.enabled() & use_segment.0);
                edge_data
                    .reverse_segment_id
                    .set_enabled(edge_data.reverse_segment_id.enabled() & use_segment.1);

                // Store phantom node in result vector:
                results.push(edge_data);
            }
        }

        results
    }

    // Iterates over all the objects in a leaf node and inserts them into our
    // search priority queue.  The speed of this function is very much governed
    // by the value of LEAF_NODE_SIZE, as we'll calculate the euclidean distance
    // for every child of each leaf node visited.
    fn explore_leaf_node(
        &self,
        traversal_queue: &mut BinaryHeap<QueryCandidate>,
        leaf_id: &TreeIndex,
        projected_input_coordinate_fixed: &Coordinate,
        projected_input_coordinate: &FloatCoordinate,
    ) {
        for index in self.child_indexes(leaf_id) {
            let edge = self.objects[index];

            let u: FloatCoordinate = self.coordinates[edge.u as usize].into();
            let v: FloatCoordinate = self.coordinates[edge.v as usize].into();
            let projected_u = u.from_wgs84();
            let projected_v = v.from_wgs84();

            let projected_nearest = crate::coordinates::project_point_on_segment(
                &projected_u,
                &projected_v,
                projected_input_coordinate,
            );

            let squared_min_dist = crate::coordinates::squared_euclidian_distance(
                &projected_input_coordinate_fixed,
                &projected_nearest.into(),
            );

            traversal_queue.push(QueryCandidate {
                squared_min_dist,
                tree_index: *leaf_id,
                segment_index: Some(index),
                fixed_projected_coordinate: Some(projected_nearest.into()),
            })
        }
    }

    // Iterates over all the children of a TreeNode and inserts them into the search
    // priority queue using their distance from the search coordinate as the
    // priority metric.
    // The closests distance to a box from our point is also the closest distance
    // to the closest line in that box (assuming the boxes hug their contents).
    fn explore_tree_node(
        &self,
        traversal_queue: &mut BinaryHeap<QueryCandidate>,
        parent: &TreeIndex,
        fixed_projected_input_coordinate: &Coordinate,
    ) {
        for child_index in self.child_indexes(parent) {
            let child = self.tree[child_index];

            let squared_min_dist = child
                .minimum_bounding_rectangle
                .get_min_squared_dist(fixed_projected_input_coordinate);

            traversal_queue.push(QueryCandidate {
                squared_min_dist,
                tree_index: TreeIndex {
                    level: parent.level + 1,
                    offset: child_index - self.tree_level_starts[parent.level + 1] as usize,
                },
                fixed_projected_coordinate: None,
                segment_index: None,
            });
        }
    }

    fn is_leaf(&self, tree_index: &TreeIndex) -> bool {
        debug_assert!(self.tree_level_starts.len() >= 2);
        return tree_index.level == self.tree_level_starts.len() - 2;
    }

    fn child_indexes(&self, parent: &TreeIndex) -> Range<usize> {
        if self.is_leaf(parent) {
            let first_child_index = parent.offset * LEAF_NODE_SIZE;
            let end_child_index =
                usize::min(first_child_index + LEAF_NODE_SIZE, self.objects.len());

            Range {
                start: first_child_index,
                end: end_child_index,
            }
        } else {
            let first_child_index = self.tree_level_starts[parent.level + 1] as usize
                + parent.offset * BRANCHING_FACTOR;
            let end_child_index = usize::min(
                first_child_index + BRANCHING_FACTOR,
                self.tree_level_starts[parent.level + 2] as usize,
            );

            Range {
                start: first_child_index,
                end: end_child_index,
            }
        }
    }
}
