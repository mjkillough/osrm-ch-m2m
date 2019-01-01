mod bindgen;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use bitvec::BitVec;
use tar;

use crate::errors::*;

// TODO: Check exclude.meta and dynamically determine EDGE_FILTER_PATH?
pub const EDGE_FILTER_PATH: &str = "/ch/metrics/duration/exclude/0/edge_filter";
pub const EDGE_ARRAY_PATH: &str = "/ch/metrics/duration/contracted_graph/edge_array";
pub const NODE_ARRAY_PATH: &str = "/ch/metrics/duration/contracted_graph/node_array";

pub type NodeId = u32;
// pub type EdgeId = u32;
pub type Weight = i32;

pub use self::bindgen::{EdgeArrayEntry, Metadata, NodeArrayEntry, Unpack};

fn read_file_from_tar(tar: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let file = BufReader::new(File::open(tar.as_ref().clone())?);
    let mut archive = tar::Archive::new(file);

    for file in archive.entries()? {
        let mut file = file?;
        if file.path()? == path.as_ref() {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            return Ok(buf);
        }
    }

    return Err(Error::MissingFile {
        tar: tar.as_ref().to_owned(),
        path: path.as_ref().to_owned(),
    });
}

fn read_metadata(tar: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<Metadata> {
    let metadata_path = path.as_ref().with_extension("meta");
    let metadata_bytes = read_file_from_tar(tar.as_ref(), metadata_path)?;
    let metadata = Metadata::unpack(&metadata_bytes);
    Ok(metadata)
}

pub fn read_array<T>(tar: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<Vec<T>>
where
    T: Unpack,
{
    let metadata = read_metadata(tar.as_ref(), path.as_ref())?;
    let element_count = metadata.element_count as usize;
    let element_size = std::mem::size_of::<T>();

    let mut vec = Vec::with_capacity(0);
    let bytes = read_file_from_tar(tar, path)?;

    for i in 0..element_count {
        let start = i * element_size;
        let element = T::unpack(&bytes[start..start + element_size]);
        vec.push(element);
    }

    // TODO: assert we read the entire file?

    Ok(vec)
}

pub fn read_bit_array(tar: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<BitVec> {
    let metadata = read_metadata(tar.as_ref(), path.as_ref())?;
    let element_count = metadata.element_count as usize;

    let bytes = read_file_from_tar(tar, path)?;
    let mut bitvec = BitVec::from(bytes);

    // There may be empty bits in the last few bytes, as the entire
    // std::vector<bool> is stored, including the empty bits in the final word.
    bitvec.truncate(element_count);

    Ok(bitvec)
}
