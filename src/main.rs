mod errors;
mod graph;
mod heap;
mod m2m;

use std::fs::File;
use std::io::BufReader;
use std::ops::Range;
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
                .map(|(_, duration)| f64::from(duration) / 10.)
                .collect()
        })
        .collect()
}

struct Problem {
    sources: Vec<(usize, Vec<heap::Query>)>,
    targets: Vec<(usize, Vec<heap::Query>)>,
    results: Vec<Vec<f64>>,
}

impl Problem {
    fn subproblem_queries(
        &self,
        source_range: Range<usize>,
        target_range: Range<usize>,
    ) -> (
        Vec<(usize, Vec<heap::Query>)>,
        Vec<(usize, Vec<heap::Query>)>,
    ) {
        (
            self.sources[source_range].to_owned(),
            self.targets[target_range].to_owned(),
        )
    }

    fn subproblem_results(
        &self,
        source_range: Range<usize>,
        target_range: Range<usize>,
    ) -> Vec<Vec<f64>> {
        self.results[source_range]
            .iter()
            .map(|row| row[target_range.clone()].to_owned())
            .collect()
    }
}

fn main() -> Result<()> {
    let queries: Queries = load_json("data/queries.json")?;
    let expected_results: Vec<Vec<f64>> = load_json("data/results.json")?;
    let graph = Graph::from_file("data/1.osrm.hsgr")?;

    let problem = Problem {
        sources: queries.sources.into_iter().enumerate().collect(),
        targets: queries.targets.into_iter().enumerate().collect(),
        results: expected_results,
    };

    let (sources, targets) = problem.subproblem_queries(0..997, 0..997);
    let expected_results = problem.subproblem_results(0..997, 0..997);

    println!("Many to Many: {} x {}", sources.len(), targets.len());
    let mut computer = m2m::ManyToMany::new(&graph, sources, targets);
    let results = time("initial", || computer.compute());
    let results = convert_results(results.clone());
    println!("Result Matrix: {} x {}", results.len(), results[0].len());
    println!("results equal expected? {}", results == expected_results);

    println!("");

    println!("Adding 3 new sources");
    let (sources, _) = problem.subproblem_queries(997..1000, 0..0);
    let expected_results = problem.subproblem_results(0..1000, 0..997);
    for source in sources {
        computer.add_source(source);
    }
    let results = time("3 new sources", || computer.compute());
    let results = convert_results(results.clone());
    println!("Result Matrix: {} x {}", results.len(), results[0].len());
    println!("results equal expected? {}", results == expected_results);

    println!("");

    println!("Adding 3 new targets");
    let (_, targets) = problem.subproblem_queries(0..0, 997..1000);
    let expected_results = problem.subproblem_results(0..1000, 0..1000);
    for target in targets {
        computer.add_target(target);
    }
    let results = time("3 new targets", || computer.compute());
    let results = convert_results(results.clone());
    println!("Result Matrix: {} x {}", results.len(), results[0].len());
    println!("results equal expected? {}", results == expected_results);

    Ok(())
}
