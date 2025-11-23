use std::path::Path;

use csv::Reader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Tag {
    pub tag_id: u64,
    pub name: String,
    pub category: u8,
    pub count: u64,
}

pub fn read_tags_from_csv(path: &Path) -> Vec<Tag> {
    let mut reader = match Reader::from_path(path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    reader
        .deserialize::<Tag>()
        .map(|result| match result {
            Ok(tag) => tag,
            Err(e) => {
                eprintln!("Bad row in CSV, check your file.\n{}", e);
                std::process::exit(1);
            }
        })
        .collect()
}
