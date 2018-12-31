#![allow(unused)]

mod coordinates;
mod errors;
mod graph;
mod heap;
mod m2m;

pub use self::coordinates::{Coordinate, FloatCoordinate};
pub use self::errors::*;
pub use self::m2m::ManyToMany;

fn main() -> Result<()> {
    let mut m2m = ManyToMany::new()?;
    m2m.perform();
    println!("Results: {:?}", m2m.results);

    Ok(())
}
