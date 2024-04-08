use std::{env, error::Error, process::{self, exit}};
use cs_analytics::glasgow_data;

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

    let glasgow_at_mouser: usize = 200 + 260 + 260;
    let glasgow_cases_at_mouser: usize = 980;
    let mut orders = glasgow_data::Orders::new(&args[1], glasgow_at_mouser, glasgow_cases_at_mouser)?;
    orders.calculate_queue();

    orders.print_stats();
    println!();

    orders.print_skipped();
    println!();

    orders.print_order_info(my_order);

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}