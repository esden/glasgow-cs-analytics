use std::error::Error;
use serde::Deserialize;

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
    #[serde(rename = "Company")]
    pub company: Option<String>,
    #[serde(rename = "Country Code")]
    pub country_code: String,
    #[serde(rename = "Placed Time")]
    pub placed_time: String,
    #[serde(rename = "Shipped Time")]
    pub shipped_time: Option<String>,
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