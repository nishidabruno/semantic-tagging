use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TagRow {
    pub tag_id: u64,
    pub name: String,
    pub category: u8,
    pub count: u64,
}

pub fn read_tags_from_csv(path: &str) -> Vec<TagRow> {
    let mut reader = csv::Reader::from_path(path).expect("Cannot open CSV file");

    reader
        .deserialize::<TagRow>()
        .filter_map(Result::ok)
        .collect()
}
