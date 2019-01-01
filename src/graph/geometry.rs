use std::ops::Range;
use std::path::Path;

use super::storage::*;
use crate::errors::*;

const INDEX_PATH: &str = "/common/segment_data/";
const FORWARD_WEIGHTS_PATH: &str = "/common/segment_data/forward_weights";
const REVERSE_WEIGHTS_PATH: &str = "/common/segment_data/reverse_weights";

pub struct Geometry {
    index: Vec<u32>,
    forward_weights: PackedVector,
    reverse_weights: PackedVector,
}

impl Geometry {
    fn from_file(tar: impl AsRef<Path>) -> Result<Geometry> {
        let index = read_array_from_tar(tar.as_ref(), INDEX_PATH)?;
        let forward_weights = read_packed_vector(tar.as_ref(), FORWARD_WEIGHTS_PATH)?;
        let reverse_weights = read_packed_vector(tar, REVERSE_WEIGHTS_PATH)?;
        Ok(Geometry {
            index,
            forward_weights,
            reverse_weights,
        })
    }

    pub fn forward_weights(&self, geometry_id: u32) -> Vec<u32> {
        let range = Range {
            start: self.index[geometry_id as usize] as usize,
            end: self.index[(geometry_id + 1) as usize] as usize,
        };
        range
            .map(|idx| self.forward_weights.get(idx) as u32)
            .collect()
    }

    pub fn reverse_weights(&self, geometry_id: u32) -> Vec<u32> {
        let range = Range {
            start: self.index[geometry_id as usize] as usize,
            end: self.index[(geometry_id + 1) as usize] as usize,
        };
        range
            .map(|idx| self.reverse_weights.get(idx) as u32)
            .collect()
    }
}
