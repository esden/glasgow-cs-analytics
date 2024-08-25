// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2024 1BitSquared <info@1bitsquared.com>
// SPDX-FileContributor: Written by Piotr Esden-Tempski <piotr@1bitsquared.com>

use core::fmt;
use std::error::Error;
use crate::order_data;
use crate::production_data;
use chrono::NaiveDate;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Product {
    Glasgow{id: usize},
    GlasgowCase{id: usize},
    GlasgowEarlyBird{id: usize},
    GlasgowCaseEarlyBird{id: usize},
    Unknown{name: String}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub cs_id: usize,
    pub date: NaiveDate,
    pub queue_id: usize,
    pub products: Vec<Product>,
    pub contains_early_bird: bool,
    pub country: String,
    pub fulfilled: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orders {
    pub orders: Vec<Order>,
    pub glasgow_count: usize,
    pub glasgow_case_count: usize,
    pub glasgow_sent_to_mouser: usize,
    pub glasgow_cases_sent_to_mouser: usize
}

impl Orders {
    pub fn new(order_data: &String, production_data: &String) -> Result<Self, Box<dyn Error>> {
        //println!("Parsing Order Data.");
        let order_data = order_data::OrderData::new(order_data)?;
        let mut orders: Vec<Order> = Vec::new();
        for record in order_data.records.iter() {
            let mut early_bird = false;
            let product =
                match record.product_name.as_str() {
                    "Glasgow revC" => Product::Glasgow { id: 0 },
                    "Glasgow revC - Early Bird" => {early_bird = true; Product::GlasgowEarlyBird{ id: 0}},
                    "Glasgow Aluminum Case" => Product::GlasgowCase { id: 0 },
                    "Glasgow Aluminum Case - Early Bird" => {early_bird = true; Product::GlasgowCaseEarlyBird { id: 0 }},
                    _ => Product::Unknown { name: record.product_name.clone() }
                };

            let shipped = record.shipped_time.is_some();

            let mut added = false;
            for order  in orders.iter_mut() {
                if order.cs_id == record.order_id {
                    for _ in 0..record.qty {
                        order.products.push(product.clone());
                    }
                    if early_bird {
                        order.contains_early_bird = true;
                    }
                    added = true;
                }
            }

            if !added {
                orders.push(Order {
                    cs_id: record.order_id,
                    date: record.placed_time.date(),
                    queue_id: 0,
                    products: vec![product; record.qty],
                    contains_early_bird: early_bird,
                    country: record.country_code.clone(),
                    fulfilled: shipped
                })
            }
        }

        //println!("Parsing production data.");
        let production_data = production_data::ProductionData::new(production_data)?;
        let mut glasgow_sent_to_mouser = 0_usize;
        let mut glasgow_cases_sent_to_mouser = 0_usize;
        for record in production_data.records.iter() {
                match record.product_name.as_str() {
                    "GLASGOW-C3" => glasgow_sent_to_mouser += record.qty,
                    "GLASGOW-C3-AL-CASE" => glasgow_cases_sent_to_mouser += record.qty,
                    _ => println!("Unknown Product {}", record.product_name.to_string())
                };
        }

        Ok(Self {
            orders,
            glasgow_count: 0,
            glasgow_case_count: 0,
            glasgow_sent_to_mouser,
            glasgow_cases_sent_to_mouser
        })
    }

    /// Sort orders by Crowd Supply order id in ascending order
    fn sort(self: &mut Self) {
        self.orders.sort_by(|a, b| a.cs_id.cmp(&b.cs_id));
    }

    /// Calculate queue IDs for a specific order type slice
    fn calculate_queue_for_slice(self: &mut Self, early_bird: bool, order_counter_start: usize) -> usize {
        let mut order_counter = order_counter_start;
        // Set order queue ID for the slice we are processing
        for o in self.orders.iter_mut().filter(|o| o.contains_early_bird == early_bird) {
            o.queue_id = order_counter;
            order_counter += 1;
        }
        // Increment product queue IDs depending on the slice we are processing
        for o in &mut self.orders {
            for p in &mut o.products {
                if early_bird {
                    match p {
                        Product::GlasgowEarlyBird { id } => {
                            *id = self.glasgow_count;
                            self.glasgow_count += 1;
                        },
                        Product::GlasgowCaseEarlyBird { id } => {
                            *id = self.glasgow_case_count;
                            self.glasgow_case_count += 1;
                        },
                        _ => continue
                    }
                } else {
                    match p {
                        Product::Glasgow { id } => {
                            *id = self.glasgow_count;
                            self.glasgow_count += 1;
                        },
                        Product::GlasgowCase { id } => {
                            *id = self.glasgow_case_count;
                            self.glasgow_case_count += 1;
                        },
                        _ => continue
                    }
                }
            }
        }
        order_counter
    }

    pub fn calculate_queue(self: &mut Self) {
        self.sort();

        // Assign queue ids to orders and products
        let queue_id = self.calculate_queue_for_slice(true, 0);
        self.calculate_queue_for_slice(false, queue_id);
    }

    pub fn get_order_count(self: &Self) -> usize {
        self.orders.len()
    }

    pub fn get_fulfilled_count(self: &Self) -> usize {
        self.orders.iter().filter(|o| o.fulfilled).count()
    }

    pub fn get_fulfilled_percent(self: &Self) -> f64 {
        ((self.get_fulfilled_count() as f64) / self.get_order_count() as f64) * 100.0
    }

    pub fn get_remaining_count(self: &Self) -> usize {
        self.orders.iter().filter(|o| !o.fulfilled).count()
    }

    pub fn get_remaining_percent(self: &Self) -> f64 {
        ((self.get_remaining_count() as f64) / self.get_order_count() as f64) * 100.0
    }

    pub fn get_fulfilled_glasgow_count(self: &Self) -> usize {
        self.orders.iter().map(|o| if o.fulfilled {
            o.products
                .iter()
                .filter(|p|
                    matches!(p, Product::Glasgow{..}) ||
                    matches!(p, Product::GlasgowEarlyBird{..}
                    ))
                .count()
        } else { 0 }).sum()
    }

    pub fn get_fulfilled_glasgow_percent(self: &Self) -> f64 {
        ((self.get_fulfilled_glasgow_count() as f64) / self.glasgow_count as f64) * 100.0
    }

    pub fn get_glasgow_at_mouser_count(self: &Self) -> i32 {
        self.glasgow_sent_to_mouser as i32 - self.get_fulfilled_glasgow_count() as i32
    }

    pub fn get_glasgow_at_mouser_percent(self: &Self) -> f64 {
        (self.get_glasgow_at_mouser_count() as f64 / self.glasgow_count as f64) * 100.0
    }

    pub fn get_remaining_glasgow_count(self: &Self) -> i32 {
        self.glasgow_count as i32 - self.glasgow_sent_to_mouser as i32
    }

    pub fn get_remaining_glasgow_percent(self: &Self) -> f64 {
        (self.get_remaining_glasgow_count() as f64 / self.glasgow_count as f64) * 100.0
    }

    pub fn get_fulfilled_glasgow_cases_count(self: &Self) -> usize {
        self.orders.iter().map(|o| if o.fulfilled {
            o.products
                .iter()
                .filter(|p|
                    matches!(p, Product::GlasgowCase{..}) ||
                    matches!(p, Product::GlasgowCaseEarlyBird{..}
                    ))
                .count()
        } else { 0 }).sum()
    }

    pub fn get_fulfilled_glasgow_cases_percent(self: &Self) -> f64 {
        ((self.get_fulfilled_glasgow_cases_count() as f64) / self.glasgow_case_count as f64) * 100.0
    }

    pub fn get_glasgow_cases_at_mouser_count(self: &Self) -> i32 {
        self.glasgow_cases_sent_to_mouser as i32 - self.get_fulfilled_glasgow_cases_count() as i32
    }

    pub fn get_glasgow_cases_at_mouser_percent(self: &Self) -> f64 {
        (((self.get_glasgow_cases_at_mouser_count()) as f64) / self.glasgow_case_count as f64) * 100.0
    }

    pub fn get_remaining_glasgow_cases_count(self: &Self) -> i32 {
        self.glasgow_case_count as i32 - self.glasgow_cases_sent_to_mouser as i32
    }

    pub fn get_remaining_glasgow_cases_percent(self: &Self) -> f64 {
        (self.get_remaining_glasgow_cases_count() as f64 / self.glasgow_case_count as f64) * 100.0
    }

    pub fn print_stats(self: &Self) {
        println!("We sent {} Glasgows and {} Glasgow Cases to Mouser.", self.glasgow_sent_to_mouser, self.glasgow_cases_sent_to_mouser);
        println!("We received {} orders, out of which {} ({:.1}%) are fulfilled.",
            self.get_order_count(),
            self.get_fulfilled_count(),
            self.get_fulfilled_percent(),
        );
        println!("The orders contain {} Glasgows, out of which {} ({:.1}%) are at Mouser and {} ({:.1}%) have shipped.",
            self.glasgow_count,
            self.get_glasgow_at_mouser_count(),
            self.get_glasgow_at_mouser_percent(),
            self.get_fulfilled_glasgow_count(),
            self.get_fulfilled_glasgow_percent(),
        );
        println!("The orders contain {} Glasgow Cases, out of which {} ({:.1}%) are at Mouser and {} ({:.1}%) have shipped.",
            self.glasgow_case_count,
            self.get_glasgow_cases_at_mouser_count(),
            self.get_glasgow_cases_at_mouser_percent(),
            self.get_fulfilled_glasgow_cases_count(),
            self.get_fulfilled_glasgow_cases_percent(),
        );
    }

    pub fn print_skipped(self: &Self) {
        println!("Here is a list of orders that have all items with queue IDs lower than the number of supplied items, but have not shipped:");
        let mut skipped_order_count = 0;
        for o in &self.orders {
            if !o.fulfilled {
                let mut can_fulfill = true;
                for p in &o.products {
                    match p {
                        Product::Glasgow { id } => if id > &self.glasgow_sent_to_mouser { can_fulfill = false },
                        Product::GlasgowCase { id } => if id > &self.glasgow_cases_sent_to_mouser { can_fulfill = false },
                        Product::GlasgowEarlyBird { id } => if id > &self.glasgow_sent_to_mouser { can_fulfill = false },
                        Product::GlasgowCaseEarlyBird { id } => if id > &self.glasgow_cases_sent_to_mouser { can_fulfill = false },
                        Product::Unknown { name: _ } => continue,
                    }
                }
                if can_fulfill {
                    println!("Order ID {}, ordered from {}", o.cs_id, o.country);
                    skipped_order_count += 1;
                }
            }
        }
        println!("Skipped order count: {}", skipped_order_count);
    }

    pub fn get_order(self: &Self, order_id: usize) -> Option<&Order> {
        self.orders.iter().find(|o| o.cs_id == order_id)
    }

    pub fn print_order_info(self: &Self, order_id: usize) {
        let order = self.get_order(order_id);

        // Order not found
        if order.is_none() {
            println!("The order ID you provided {} was not found!", order_id);
            return;
        }

        let order = order.unwrap();

        println!("Your order number {} was placed on {} and has the queue id {}.", order_id, order.date, order.queue_id);

        // Order fulfilled
        if order.fulfilled {
            println!("Your order was fulfilled.");
            return;
        }

        // Found order, print info
        println!("We have fulfilled {} orders, so there are still {} orders to fulfill before it is your turn.",
            self.get_fulfilled_count(),
            order.queue_id as i64 - self.get_fulfilled_count() as i64
        );
        if (order.queue_id as i64 - self.get_fulfilled_count() as i64) < 0 {
            println!("According to our records your order should have already shipped, there might be a lot of reasons why this is the case.");
            println!("Here are a few potential reasons:");
            println!(" * You are located in a country that got sanctioned after the campaign happened, for example Russian Federation.");
            println!(" * You had or have pending fees on your order due to customs cost adjustments and your queue position had to be pushed.");
            println!(" * Our queue placement prediction does not match reality. You can contact Crowd Supply directly and ask for the reason why your order was not shipped yet.");
        }
        println!("Your order contains:");
        for p in &order.products {
            match p {
                Product::Glasgow { id } => {
                    print!("- Glasgow with the queue ID {}, ", id);
                    if id <= &self.glasgow_sent_to_mouser {
                        println!("it is at Mouser and will ship soon, if all items in your order are available.")
                    } else {
                        println!("we have to ship {} Glasgows to Mouser before your order can be fulfilled.", id - self.glasgow_sent_to_mouser)
                    }
                },
                Product::GlasgowCase { id } => {
                    print!("- Glasgow Case with the queue ID {}, ", id);
                    if id <= &self.glasgow_cases_sent_to_mouser {
                        println!("it is at Mouser and will ship soon, if all items in your order are available.")
                    } else {
                        println!("we have to ship {} cases to Mouser before your order can be fulfilled.", id - self.glasgow_cases_sent_to_mouser)
                    }
                },
                Product::GlasgowEarlyBird { id } => {
                    print!("- EarlyBird Glasgow with the queue ID {}, ", id);
                    if id <= &self.glasgow_sent_to_mouser {
                        println!("it is at Mouser and will ship soon, if all items in your order are available.")
                    } else {
                        println!("we have to ship {} Glasgows to Mouser before your order can be fulfilled.", id - self.glasgow_sent_to_mouser)
                    }
                },
                Product::GlasgowCaseEarlyBird { id } => {
                    print!("- EarlyBird Glasgow Case with the queue ID {}, ", id);
                    if id <= &self.glasgow_cases_sent_to_mouser {
                        println!("it is at Mouser and will ship soon, if all items in your order are available.")
                    } else {
                        println!("we have to ship {} cases to Mouser before your order can be fulfilled.", id - self.glasgow_cases_sent_to_mouser)
                    }
                },
                Product::Unknown { name } => println!("- Unknown Product with the name \"{}\".", name),
            }
        }
    }
}

impl fmt::Display for Orders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "order data")
    }
}