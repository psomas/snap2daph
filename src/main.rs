use std::{
    collections::HashSet,
    fs::File,
    io::{prelude::*, BufReader, BufWriter},
};

use clap::Parser;
use itertools::Itertools;
use serde::Serialize;
use serde_json;

mod df;

const DELIMITERS: &str = ";, \t";

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    input: String,

    #[arg(long)]
    output: String,

    /* WIP: */
    #[arg(long, value_name = "SCALING FACTOR")]
    scale: Option<usize>,

    /* WIP: */
    #[arg(long, value_name = "DENSIFICATION_FACTOR")]
    densify: Option<usize>,

    #[arg(long)]
    undirected: bool,

    #[arg(long)]
    daphne_serde: bool,
}

#[derive(Serialize, Clone)]
struct Metadata {
    numRows: usize,
    numCols: usize,
    valueType: String,
    numNonZeros: usize,
}

#[derive(Eq, Hash, PartialEq, Ord, PartialOrd, Copy, Clone)]
struct Vertex(usize);

impl From<Vertex> for usize {
    fn from(v: Vertex) -> Self {
        v.0
    }
}

#[derive(Eq, Hash, PartialEq, Ord, PartialOrd, Copy, Clone)]
struct Edge(Vertex, Vertex);

impl Edge {
    fn rev(&self) -> Self {
        Self(self.1, self.0)
    }
}

impl From<(usize, usize)> for Edge {
    fn from(tuple: (usize, usize)) -> Self {
        Edge(Vertex(tuple.0), Vertex(tuple.1))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input = BufReader::new(File::open(args.input)?);
    let mut min = usize::MAX;
    let mut max = 0;
    let mut edges: HashSet<Edge> = HashSet::new();
    let mut vertices: HashSet<Vertex> = HashSet::new();

    for line in input.lines() {
        let line = line?;
        let line = line.trim();

        if line.starts_with("#") {
            continue;
        }

        let edge: (usize, usize) = line
            .split(|c| DELIMITERS.contains(c))
            .map(|v| {
                let v = v.parse::<usize>().unwrap();
                vertices.insert(Vertex(v));
                if v < min {
                    min = v;
                }
                if v > max {
                    max = v;
                }
                v
            })
            .collect_tuple()
            .ok_or("error parsing file")?;

        let edge = edge.into();
        edges.insert(edge);
        if args.undirected {
            edges.insert(edge.rev());
        }
    }

    let mut vertices: Vec<_> = vertices.into_iter().collect();
    vertices.sort();

    let mut edges: Vec<_> = edges.into_iter().collect();
    edges.sort();

    let meta = Metadata {
        numRows: max - min,
        numCols: max - min,
        valueType: "f64".to_owned(),
        numNonZeros: edges.len(),
    };

    let mut meta_out = BufWriter::new(File::create(format!("{}.meta", args.output))?);
    write!(meta_out, "{}", serde_json::to_string(&meta)?)?;

    let mut out = BufWriter::new(File::create(&args.output)?);
    if args.daphne_serde {
        let ds = df::Serializer {
            vertices,
            edges,
            meta,
        };
        ds.serialize(&mut out)?;
    } else {
        for edge in &edges {
            write!(
                out,
                "{},{}\n",
                usize::from(edge.0) - min,
                usize::from(edge.1) - min
            )?;
        }
    }

    Ok(())
}
