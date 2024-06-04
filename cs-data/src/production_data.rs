// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2024 1BitSquared <info@1bitsquared.com>
// SPDX-FileContributor: Written by Piotr Esden-Tempski <piotr@1bitsquared.com>

use std::error::Error;
use serde::Deserialize;
use chrono::NaiveDate;

mod date_deserializer {
    use serde::{de::Error, Deserializer};
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDate, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        Ok(NaiveDate::parse_from_str(&time, "%m/%d/%Y").map_err(D::Error::custom)?)
    }

}

/// Crowd Supply order data record
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Record {
    #[serde(rename = "Date", deserialize_with = "date_deserializer::deserialize")]
    pub date: NaiveDate,
    #[serde(rename = "Order No")]
    pub order_no: String,
    #[serde(rename = "Box")]
    pub box_no: usize,
    #[serde(rename = "Mouser PN")]
    pub part_number: String,
    #[serde(rename = "Vendor PN")]
    pub product_name: String,
    #[serde(rename = "QTY")]
    pub qty: usize,
}

/// Deserialized Crowd Supply order data
pub struct ProductionData {
    pub records: Vec<Record>
}

impl ProductionData {
    pub fn new(data: &String) -> Result<Self, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(data).unwrap();
        let records = rdr.deserialize().collect::<Result<Vec<Record>, csv::Error>>()?;
        Ok(Self {
            records
        })
    }
}