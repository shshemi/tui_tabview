use std::cmp::max;
use std::fs::File;
use std::io::BufReader;
use std::ops::Index;
use std::path::Path;
use csv::{Error, Reader, StringRecord};
use crate::model_table::DataSource;


pub struct CSV {
    v: Vec<StringRecord>,
    w: Vec<u16>,
}

impl CSV {
    pub fn new(path: &impl AsRef<Path>) -> Self {
        let mut reader = match Reader::from_path(path) {
            Ok(result) => { result }
            Err(err) => { panic!("Error: {}", err) }
        };
        let mut v = Vec::new();
        let mut w = Vec::new();
        for rec in reader.records() {
            let rec = rec.unwrap();
            if w.is_empty() {
                for e in rec.iter() {
                    w.push(e.len() as u16)
                }
            } else {
                for (l, e) in rec.iter().enumerate() {
                    w[l] = max(w[l], e.len() as u16);
                }
            }
            v.push(rec);
        }
        CSV {
            v,
            w,
        }
    }
}

impl DataSource for CSV {
    fn value(&self, row: usize, col: usize) -> &str {
        self.v[row].index(col)
    }

    fn shape(&self) -> (usize, usize) {
        (self.v.len(), self.v.first().unwrap().len())
    }

    fn max_widths(&self) -> &[u16] {
        &self.w
    }
}