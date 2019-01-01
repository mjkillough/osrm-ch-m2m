#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

use super::Unpack;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl Unpack for EdgeArrayEntry {}
impl Unpack for NodeArrayEntry {}
impl Unpack for Metadata {}
