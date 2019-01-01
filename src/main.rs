mod errors;
mod graph;
mod heap;
mod m2m;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::{de::DeserializeOwned, Deserialize};
use serde_json;

pub use self::errors::*;
use self::graph::Graph;

#[derive(Deserialize)]
struct Queries {
    sources: Vec<Vec<heap::Query>>,
    targets: Vec<Vec<heap::Query>>,
}

fn load_json<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: DeserializeOwned,
{
    let file = BufReader::new(File::open(path)?);
    let deserialized = serde_json::from_reader(file)?;
    Ok(deserialized)
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

fn convert_results(results: Vec<Vec<Option<(i32, i32)>>>) -> Vec<Vec<f64>> {
    results
        .iter()
        .map(|row| {
            row.iter()
                .map(|option| option.unwrap_or((0, 0)))
                // Durations are in deciseconds (why??)
                .map(|(_, duration)| duration as f64 / 10.)
                .collect()
        })
        .collect()
}

fn main() -> Result<()> {
    let queries: Queries = load_json("data/queries.json")?;
    let expected_results: Vec<Vec<f64>> = load_json("data/results.json")?;
    let graph = Graph::from_file("data/1.osrm.hsgr")?;

    println!(
        "Many to Many: {} x {}",
        queries.sources.len(),
        queries.targets.len()
    );

    let results = time("non-parallel", || {
        m2m::many_to_many(&graph, queries.sources.clone(), queries.targets.clone())
    });
    println!(
        "non-parallel results equal expected? {}",
        convert_results(results) == expected_results
    );

    let results = time("parallel", || {
        m2m::parallel_many_to_many(&graph, queries.sources, queries.targets)
    });
    println!(
        "non-parallel results equal expected? {}",
        convert_results(results) == expected_results
    );

    println!("Used {} threads", rayon::current_num_threads());

    Ok(())
}
