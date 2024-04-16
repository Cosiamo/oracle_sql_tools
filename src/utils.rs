use itertools::Itertools;

use crate::{format_data::FormattedData, types::DatatypeIndexes};

/// Removes characters that are invalid in SQL column names
pub fn remove_invalid_chars(input: &String) -> String {
    input
        .trim()
        .replace(|c: char| !c.is_ascii(), "")
        .replace(" ", "_")
        .replace("-", "_")
        .replace("'", "")
        .replace("%", "")
        .replace("!", "")
        .replace("?", "")
        .replace("|", "")
        .replace("#", "")
        .replace("\\", "")
        .replace("/", "")
        .replace("(", "")
        .replace(")", "")
        .replace("+", "")
        .replace("#", "")
}

impl FormattedData {
    pub(crate) fn to_string(self) -> String {
        match self {
            FormattedData::STRING(val) => val.to_owned(),
            FormattedData::INT(val) => val.to_string(),
            FormattedData::FLOAT(val) => val.to_string(),
            FormattedData::DATE(val) => val.to_string(),
            FormattedData::EMPTY => "".to_string(),
        }
    }
}

impl DatatypeIndexes {
    pub(crate) fn find_uniques(mut self) -> Self {
        let is_varchar = self.is_varchar.into_iter().unique().collect::<Vec<usize>>();
        for x_index in is_varchar.iter() {
            if self.is_float.contains(x_index) { self.is_float.retain(|v| *v != *x_index); }
            else if self.is_int.contains(x_index) { self.is_int.retain(|v| *v != *x_index); }
            else if self.is_date.contains(x_index) { self.is_date.retain(|v| *v != *x_index); }
            else { continue }
        };
        let is_float = self.is_float.into_iter().unique().collect::<Vec<usize>>();
        for x_index in is_float.iter() {
            if self.is_int.contains(x_index) { self.is_int.retain(|v| *v != *x_index); }
            else if self.is_date.contains(x_index) { self.is_date.retain(|v| *v != *x_index); }
            else { continue }
        }
        let is_int = self.is_int.into_iter().unique().collect::<Vec<usize>>();
        for x_index in is_int.iter() {
            if self.is_date.contains(x_index) { self.is_date.retain(|v| *v != *x_index); }
            else { continue }
        }
        let is_date = self.is_date.into_iter().unique().collect::<Vec<usize>>();
        Self {
            is_varchar,
            is_float,
            is_int,
            is_date,
        }
    }
}