#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::mem::{size_of, transmute};

use byteorder::{ByteOrder, NativeEndian};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub trait Unpack {
    fn unpack(bytes: &[u8]) -> Self;
}

macro_rules! derive_unpack {
    ($ty:ident) => {
        impl Unpack for $ty {
            fn unpack(bytes: &[u8]) -> Self {
                assert!(bytes.len() == size_of::<$ty>());
                let mut array: [u8; size_of::<$ty>()] = Default::default();
                array.copy_from_slice(bytes);
                unsafe { transmute(array) }
            }
        }
    };
}

// Graph
derive_unpack!(EdgeArrayEntry);
derive_unpack!(NodeArrayEntry);
derive_unpack!(Metadata);

// RTree
derive_unpack!(TreeNode);
derive_unpack!(TreeLevelStart);
derive_unpack!(Coordinate);

// fileIndex
derive_unpack!(EdgeBasedNodeSegment);

derive_unpack!(GeometryID);
derive_unpack!(ComponentID);
derive_unpack!(EdgeBasedNode);

derive_unpack!(SegmentIndex);

impl From<Coordinate> for crate::Coordinate {
    fn from(other: Coordinate) -> crate::Coordinate {
        crate::Coordinate {
            longitude: crate::coordinates::FixedLongitude(other.longitude),
            latitude: crate::coordinates::FixedLatitude(other.latitude),
        }
    }
}
