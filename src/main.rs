use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

use clap::Parser;
use itertools::Itertools;
use serde_json::json;

const DELIMITERS: &str = ";, \t";

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    input: String,

    #[arg(long)]
    output: String,

    #[arg(long, value_name = "INFLATION_FACTOR")]
    inflate: Option<usize>,

    #[arg(long, value_name = "DENSIFICATION_FACTOR")]
    densify: Option<usize>,

    #[arg(long)]
    undirected: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input = BufReader::new(File::open(args.input)?);
    let mut min_vertex = usize::MAX;
    let mut edges: HashSet<(usize, usize)> = HashSet::new();
    let mut vertices = HashSet::new();

    for line in input.lines() {
        let line = line?;
        let line = line.trim();

        if line.starts_with("#") {
            continue;
        }

        let edge = line
            .split(|c| DELIMITERS.contains(c))
            .map(|v| {
                let v = v.parse::<usize>().unwrap();
                vertices.insert(v);
                if v < min_vertex {
                    min_vertex = v;
                }
                v
            })
            .collect_tuple()
            .ok_or("error parsing file")?;

        edges.insert(edge);
        if args.undirected {
            edges.insert((edge.1, edge.0));
        }
    }

    let mut edges: Vec<_> = edges.iter().collect();
    edges.sort();

    let mut output = BufWriter::new(File::create(&args.output)?);
    for edge in &edges {
        write!(output, "{},{}\n", edge.0 - min_vertex, edge.1 - min_vertex)?;
    }

    let mut output_meta = BufWriter::new(File::create(format!("{}.meta", args.output))?);
    let meta = json!({
        "numRows": vertices.len(),
        "numCols": vertices.len(),
        "valueType": "f64",
        "numNonZeros": edges.len(),
    });
    write!(output_meta, "{}", meta.to_string())?;

    Ok(())
}
