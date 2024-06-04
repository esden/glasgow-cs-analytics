// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2024 1BitSquared <info@1bitsquared.com>
// SPDX-FileContributor: Written by Piotr Esden-Tempski <piotr@1bitsquared.com>

use std::{env, error::Error, process::{self, exit}};
use cs_data::glasgow_data;

fn run() -> Result<(), Box<dyn Error>> {

    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("You need to provide the mouser fulfillment data CSV, production data CSV and an order ID as a command line parameters!");
        exit(1);
    }
    let my_order = args[3].parse::<usize>();
    if my_order.is_err() {
        println!("Could not parse the order ID you provided {} as usize number. {:?}", args[3], my_order.err());
        exit(1);
    }
    let my_order = my_order.unwrap();

    let mut orders = glasgow_data::Orders::new(&args[1], &args[2])?;
    orders.calculate_queue();

    orders.print_stats();
    println!();

    // orders.print_skipped();
    // println!();

    orders.print_order_info(my_order);

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}