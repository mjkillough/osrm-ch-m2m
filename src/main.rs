mod errors;
mod graph;
mod heap;
mod m2m;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::Deserialize;
use serde_json;

pub use self::errors::*;
use self::graph::Graph;
pub use self::m2m::ManyToMany;

#[derive(Deserialize)]
struct Queries {
    sources: Vec<Vec<heap::Query>>,
    targets: Vec<Vec<heap::Query>>,
}

fn load_queries(path: impl AsRef<Path>) -> Result<Queries> {
    let file = BufReader::new(File::open(path)?);
    let queries = serde_json::from_reader(file)?;
    Ok(queries)
}

fn time<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = f();
    let end = std::time::Instant::now();
    println!("Timing {} took: {:?}", name, end - start);
    result
}

fn main() -> Result<()> {
    let queries = load_queries("data/queries.json")?;
    let graph = Graph::from_file("data/1.osrm.hsgr")?;

    let mut m2m = ManyToMany::new(graph, queries.targets, queries.sources)?;

    time("m2m.perform()", || m2m.perform());

    println!("Results: {:?}", m2m.results.len());

    Ok(())
}
