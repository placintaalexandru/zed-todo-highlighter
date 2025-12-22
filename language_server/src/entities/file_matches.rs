use std::collections::HashMap;

use getset::Getters;

use crate::entities::{Column, Row, RowMetadata};

#[derive(Debug, Getters)]
pub struct Match {
    #[getset(get = "pub")]
    column: Column,
    #[getset(get = "pub")]
    keyword: String,
}

impl Match {
    pub fn new(column: Column, matched_patch: String) -> Self {
        Self {
            column,
            keyword: matched_patch,
        }
    }
}

#[derive(Debug, Default, Getters)]
pub struct FileState {
    #[getset(get = "pub")]
    rows: HashMap<Row, (RowMetadata, Vec<Match>)>,
}

impl FileState {
    pub fn try_new(data: HashMap<Row, (RowMetadata, Vec<Match>)>) -> Option<Self> {
        (!data.is_empty()).then_some(Self { rows: data })
    }

    pub fn replace(&mut self, row: Row, metadata: RowMetadata, new_matches: Vec<Match>) {
        self.rows.insert(row, (metadata, new_matches));
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.rows.len()
    }
}
