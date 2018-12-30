#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::mem::{size_of, transmute};

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

derive_unpack!(EdgeArrayEntry);
derive_unpack!(NodeArrayEntry);
derive_unpack!(Metadata);
