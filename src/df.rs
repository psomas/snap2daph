use std::io::prelude::*;
use std::io::BufWriter;

use std::str::FromStr;
use strum_macros::{EnumString, FromRepr};

use zerocopy::AsBytes;

use crate::{Edge, Metadata, Vertex};

#[repr(u8)]
#[derive(FromRepr, EnumString, AsBytes)]
#[strum(ascii_case_insensitive)]
enum ValueType {
    SI8 = 0,
    SI32,
    SI64,
    UI8,
    UI32,
    UI64,
    F32,
    F64,
    INVALID,
}

#[repr(u8)]
#[derive(AsBytes)]
enum DataType {
    reserved = 0,
    DenseMatrix_t,
    CSRMatrix_t,
    Frame_t,
    Value_t,
}

#[repr(packed)]
#[derive(AsBytes)]
struct Header {
    version: u8,
    dt: DataType,
    nbrows: u64,
    nbcols: u64,
}

#[repr(packed)]
#[derive(AsBytes)]
struct Body {
    rx: u64,
    cx: u64,
}

#[repr(u8)]
#[derive(AsBytes)]
enum BodyType {
    empty = 0,
    dense,
    sparse,
    ultra_sparse,
}

#[repr(packed)]
#[derive(AsBytes)]
struct BodyBlock {
    nbrows: u32,
    nbcols: u32,
    bt: BodyType,
}

pub struct Serializer {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub meta: Metadata,
}

impl Serializer {
    pub fn serialize<W: Write>(
        &self,
        writer: &mut BufWriter<W>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let header = Header {
            version: 1,
            dt: DataType::CSRMatrix_t,
            nbrows: self.meta.numRows as u64,
            nbcols: self.meta.numCols as u64,
        };
        writer.write_all(header.as_bytes())?;
        writer.write_all(ValueType::from_str(&self.meta.valueType)?.as_bytes())?;

        let body = Body { rx: 0, cx: 0 };
        writer.write_all(body.as_bytes())?;

        let block = BodyBlock {
            nbrows: self.meta.numRows as u32,
            nbcols: self.meta.numCols as u32,
            bt: BodyType::sparse,
        };
        writer.write_all(block.as_bytes())?;
        writer.write_all(ValueType::from_str(&self.meta.valueType)?.as_bytes())?;
        writer.write_all(&self.meta.numNonZeros.to_ne_bytes())?;

        let mut values = vec![];
        let mut cols = vec![];
        let mut rows = vec![0];

        let mut row = 0;
        let mut offset = 0;

        self.edges.iter().for_each(|edge| {
            let Vertex(from) = edge.0;
            let Vertex(to) = edge.1;

            values.push(1.0);
            cols.push(to);

            if from != row {
                rows.push(offset);
                row = from;
            }

            offset += 1;
        });

        writer.write_all(rows.as_bytes())?;
        writer.write_all(cols.as_bytes())?;
        writer.write_all(values.as_bytes())?;

        Ok(())
    }
}
