use std::error::Error;
use serde::Deserialize;
use chrono::NaiveDateTime;

mod date_deserializer {
    use serde::{de::Error, Deserializer};
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDateTime, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        Ok(NaiveDateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S%.f").map_err(D::Error::custom)?)
    }

    pub fn opt_deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        if time.is_empty() {
            Ok(None)
        } else {
            Ok(Some(NaiveDateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S%.f").map_err(D::Error::custom)?))
        }
    }

}

/// Crowd Supply order data record
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct Record {
    #[serde(rename = "Order ID")]
    pub order_id: usize,
    #[serde(rename = "Part Number")]
    pub part_number: String,
    #[serde(rename = "Product Name")]
    pub product_name: String,
    #[serde(rename = "Qty")]
    pub qty: usize,
    #[serde(rename = "Unit Price")]
    pub unit_price: f32,
    #[serde(rename = "Subtotal")]
    pub subtotal: f32,
    #[serde(rename = "Company")]
    pub company: Option<String>,
    #[serde(rename = "Country Code")]
    pub country_code: String,
    #[serde(rename = "Placed Time", deserialize_with = "date_deserializer::deserialize")]
    pub placed_time: NaiveDateTime,
    #[serde(rename = "Shipped Time", deserialize_with = "date_deserializer::opt_deserialize")]
    pub shipped_time: Option<NaiveDateTime>,
    #[serde(rename = "Tracking")]
    pub tracking: Option<String>
}

/// Deserialized Crowd Supply order data
pub(crate) struct OrderData {
    pub records: Vec<Record>
}

impl OrderData {
    pub fn new(data: &String) -> Result<Self, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(data).unwrap();
        let records = rdr.deserialize().collect::<Result<Vec<Record>, csv::Error>>()?;
        Ok(Self {
            records
        })
    }
}