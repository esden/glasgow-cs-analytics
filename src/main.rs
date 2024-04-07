use std::{env, error::Error, process::{self, exit}};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
enum PartNumber {
    #[serde(rename = "GLASGOW-C3")]
    Glasgow,
    #[serde(rename = "GLASGOW-C3-AL-CASE")]
    GlasgowCase
}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "Order ID")]
    order_id: usize,
    #[serde(rename = "Part Number")]
    _part_number: PartNumber,
    #[serde(rename = "Product Name")]
    product_name: String,
    #[serde(rename = "Qty")]
    qty: usize,
    #[serde(rename = "Company")]
    _company: Option<String>,
    #[serde(rename = "Country Code")]
    country_code: String,
    #[serde(rename = "Placed Time")]
    _placed_time: String,
    #[serde(rename = "Shipped Time")]
    shipped_time: Option<String>,
    #[serde(rename = "Tracking")]
    _tracking: Option<String>
}

#[derive(Debug, Clone)]
enum Product {
    Glasgow{id: usize},
    GlasgowCase{id: usize},
    GlasgowEarlyBird{id: usize},
    GlasgowCaseEarlyBird{id: usize},
    Unknown{name: String}
}

#[derive(Debug)]
struct Order {
    cs_id: usize,
    queue_id: usize,
    products: Vec<Product>,
    country: String,
    fulfilled: bool
}

fn run() -> Result<(), Box<dyn Error>> {

    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("You need to provide the data CSV and an order ID as a command line parameter!");
        exit(1);
    }
    let my_order = args[2].parse::<usize>();
    if my_order.is_err() {
        println!("Could not parse the order ID you provided {} as usize number. {:?}", args[2], my_order.err());
        exit(1);
    }
    let my_order = my_order.unwrap();

    // Read parse and decode the order data
    let mut rdr = csv::Reader::from_path(&args[1]).unwrap();
    let mut orders: Vec<Order> = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;

        let product = {
            match record.product_name.as_str() {
                "Glasgow revC" => Product::Glasgow { id: 0 },
                "Glasgow revC - Early Bird" => Product::GlasgowEarlyBird{ id: 0},
                "Glasgow Aluminum Case" => Product::GlasgowCase { id: 0 },
                "Glasgow Aluminum Case - Early Bird" => Product::GlasgowCaseEarlyBird { id: 0 },
                _ => Product::Unknown { name: record.product_name }
            }
        };

        let shipped = record.shipped_time.is_some();

        let mut added = false;
        for order in orders.iter_mut() {
            if order.cs_id == record.order_id {
                for _ in 0..record.qty {
                    order.products.push(product.clone());
                }
                added = true;
            }
        }

        if !added {
            orders.push(Order { cs_id: record.order_id, queue_id: 0, products: vec![product; record.qty], country: record.country_code, fulfilled: shipped })
        }
    }

    // Sort orders by crowd supply order id in ascending order
    orders.sort_by(|a, b| a.cs_id.cmp(&b.cs_id));


    // Assign queue ids to orders and products
    let mut order_counter = 0_usize;
    let mut glasgow_counter = 0_usize;
    let mut glasgow_case_counter = 0_usize;
    // Early bird order and product queue id assignment
    for o in &mut orders {
        let mut early_bird = false;
        for p in &mut o.products {
            match p {
                Product::Glasgow { id: _ } => continue,
                Product::GlasgowCase { id: _ } => continue,
                Product::GlasgowEarlyBird { id } => {
                    *id = glasgow_counter;
                    glasgow_counter += 1;
                    early_bird = true;
                },
                Product::GlasgowCaseEarlyBird { id } => {
                    *id = glasgow_case_counter;
                    glasgow_case_counter += 1;
                    early_bird = true;
                },
                Product::Unknown { name: _ } => continue,
            }
        }
        if early_bird {
            o.queue_id = order_counter;
            order_counter += 1;
        }
    }

    // General order and product queue id assignment
    for o in &mut orders {
        let mut early_bird = false;
        for p in &mut o.products {
            match p {
                Product::Glasgow { id } => {
                    *id = glasgow_counter;
                    glasgow_counter += 1;
                },
                Product::GlasgowCase { id } => {
                    *id = glasgow_case_counter;
                    glasgow_case_counter += 1;
                },
                Product::GlasgowEarlyBird { id: _ } => early_bird = true,
                Product::GlasgowCaseEarlyBird { id: _ } => early_bird = true,
                Product::Unknown { name: _ } => continue,
            }
        }
        if !early_bird {
            o.queue_id = order_counter;
            order_counter += 1;
        }
    }

    let mut fullfilled_count = 0_usize;
    let mut glasgow_count = 0_usize;
    let mut glasgow_fulfilled_count = 0_usize;
    let mut glasgow_case_count = 0_usize;
    let mut glasgow_case_fulfilled_count = 0_usize;
    for o in &orders {
        if o.fulfilled {
            fullfilled_count += 1;
        }
        for p in &o.products {
            match p {
                Product::Glasgow { id: _ } => {
                    glasgow_count += 1;
                    if o.fulfilled {
                        glasgow_fulfilled_count += 1;
                    }
                },
                Product::GlasgowCase { id: _ } => {
                    glasgow_case_count += 1;
                    if o.fulfilled {
                        glasgow_case_fulfilled_count += 1;
                    }
                },
                Product::GlasgowEarlyBird { id: _ } => {
                    glasgow_count += 1;
                    if o.fulfilled {
                        glasgow_fulfilled_count += 1;
                    }
                },
                Product::GlasgowCaseEarlyBird { id: _ } => {
                    glasgow_case_count += 1;
                    if o.fulfilled {
                        glasgow_case_fulfilled_count += 1;
                    }
                },
                Product::Unknown { name: _ } => continue,
            }
        }
    }

    let glasgow_at_mouser: usize = 200 + 260 + 260;
    let glasgow_cases_at_mouser: usize = 980;

    println!("We have {} orders, out of which {} ({:.1}%) are fulfilled.",
        order_counter,
        fullfilled_count,
        ((fullfilled_count as f64) / order_counter as f64) * 100.0,
    );
    println!("We have {} Glasgows ordered, out of which {} ({:.1}%) are at Mouser and {} ({:.1}%) have shipped.",
        glasgow_count,
        glasgow_at_mouser as i32 - glasgow_fulfilled_count as i32,
        (((glasgow_at_mouser as i32 - glasgow_fulfilled_count as i32) as f64) / glasgow_count as f64) * 100.0,
        glasgow_fulfilled_count,
        ((glasgow_fulfilled_count as f64) / glasgow_count as f64) * 100.0,
    );
    println!("We have {} Glasgow Cases ordered, out of which {} ({:.1}%) are at Mouser and {} ({:.1}%) have shipped.",
        glasgow_case_count,
        glasgow_cases_at_mouser - glasgow_case_fulfilled_count,
        (((glasgow_cases_at_mouser - glasgow_case_fulfilled_count) as f64) / glasgow_case_count as f64) * 100.0,
        glasgow_case_fulfilled_count,
        ((glasgow_case_fulfilled_count as f64) / glasgow_case_count as f64) * 100.0,
    );

    println!();

    println!("Here is a list of orders that have all items with queue IDs lower than the number of supplied items, but have not shipped:");
    let mut skipped_order_count = 0;
    for o in &orders {
        if !o.fulfilled {
            let mut can_fulfill = true;
            for p in &o.products {
                match p {
                    Product::Glasgow { id } => if id > &glasgow_at_mouser { can_fulfill = false },
                    Product::GlasgowCase { id } => if id > &glasgow_cases_at_mouser { can_fulfill = false },
                    Product::GlasgowEarlyBird { id } => if id > &glasgow_at_mouser { can_fulfill = false },
                    Product::GlasgowCaseEarlyBird { id } => if id > &glasgow_cases_at_mouser { can_fulfill = false },
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

    println!();

    let mut found = false;
    for o in &orders {
        if o.cs_id == my_order {
            found = true;
            if o.fulfilled {
                println!("Your order was fulfilled.");
            } else {
                println!("Your order number {} has the queue id {}.", my_order, o.queue_id);
                println!("We have fulfilled {} orders, so there are still {} orders to fulfill before it is your turn.",
                    fullfilled_count,
                    o.queue_id - fullfilled_count
                );
                println!("Your order contains:");
                for p in &o.products {
                    match p {
                        Product::Glasgow { id } => {
                            print!("- Glasgow with the queue ID {}, ", id);
                            if id <= &glasgow_at_mouser {
                                println!("it is at Mouser and will ship soon, if all items in your order are available.")
                            } else {
                                println!("we have to ship {} Glasgows to Mouser before your order can be fulfilled.", id - glasgow_at_mouser)
                            }
                        },
                        Product::GlasgowCase { id } => {
                            print!("- Glasgow Case with the queue ID {}, ", id);
                            if id <= &glasgow_cases_at_mouser {
                                println!("it is at Mouser and will ship soon, if all items in your order are available.")
                            } else {
                                println!("we have to ship {} cases to Mouser before your order can be fulfilled.", id - glasgow_cases_at_mouser)
                            }
                        },
                        Product::GlasgowEarlyBird { id } => {
                            print!("- EarlyBird Glasgow with the queue ID {}, ", id);
                            if id <= &glasgow_at_mouser {
                                println!("it is at Mouser and will ship soon, if all items in your order are available.")
                            } else {
                                println!("we have to ship {} Glasgows to Mouser before your order can be fulfilled.", id - glasgow_at_mouser)
                            }
                        },
                        Product::GlasgowCaseEarlyBird { id } => {
                            print!("- EarlyBird Glasgow Case with the queue ID {}, ", id);
                            if id <= &glasgow_cases_at_mouser {
                                println!("it is at Mouser and will ship soon, if all items in your order are available.")
                            } else {
                                println!("we have to ship {} cases to Mouser before your order can be fulfilled.", id - glasgow_cases_at_mouser)
                            }
                        },
                        Product::Unknown { name } => println!("- Unknown Product with the name \"{}\".", name),
                    }
                }
            }
        }
    }

    if !found {
        println!("The order ID you provided {} was not found!", my_order);
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}