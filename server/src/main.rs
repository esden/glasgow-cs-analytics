// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2024 1BitSquared <info@1bitsquared.com>
// SPDX-FileContributor: Written by Piotr Esden-Tempski <piotr@1bitsquared.com>

use anyhow::Context;
use askama::Template;
use axum::{
    extract::{Query, State}, http::StatusCode, response::{Html, IntoResponse, Response}, routing::get, Router
};
use axum_server;
use serde::Deserialize;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use cs_data::glasgow_data;
use chrono::NaiveDate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("reading and analyzing data...");

    let fulfillment_data = std::env::var("FULFILLMENT_DATA").unwrap();
    let production_data = std::env::var("PRODUCTION_DATA").unwrap();

    info!("Fulfillment data path: {:?}", fulfillment_data);
    info!("Production data path: {:?}", production_data);

    let mut orders = glasgow_data::Orders::new(&fulfillment_data, &production_data).unwrap();
    orders.calculate_queue();
    orders.print_stats();

    info!("initializing router...");

    let router = Router::new()
        .route("/", get(index_page))
        .route("/order", get(order_page))
        .with_state(orders);
    let port = 8019_u16;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    info!("router initialized, now listening on port {}", port);

    axum_server::Server::bind(addr)
        .serve(router.into_make_service())
        .await
        .context("error while starting server")?;

    Ok(())
}

async fn index_page(State(orders): State<glasgow_data::Orders>) -> impl IntoResponse {
    info!("Index page call.");
    let template =
        IndexTemplate {orders};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    orders: glasgow_data::Orders
}

#[derive(Deserialize)]
struct OrderQuery {
    id: usize,
    year: i32,
    month: u32,
    day: u32,
}

async fn order_page(State(orders): State<glasgow_data::Orders>, Query(order_query): Query<OrderQuery>) -> impl IntoResponse {
    let mut order = orders.get_order(order_query.id).cloned();
    let query_date = NaiveDate::from_ymd_opt(order_query.year, order_query.month, order_query.day);
    if order.is_some() {
        info!("Query of order id {} with date {}-{}-{}. -> Valid (Date {}) https://glasgow.1bitsquared.com/order?id={}&year={}&month={}&day={}",
            order_query.id, order_query.year, order_query.month, order_query.day,
            order.clone().unwrap().date,
            order_query.id, order_query.year, order_query.month, order_query.day
        );
    } else {
        info!("Query of order id {} with date {}-{}-{}. -> Invalid", order_query.id, order_query.year, order_query.month, order_query.day);
    }
    order = if order.is_some() && query_date.is_some() {
            let od = order.unwrap();
            let diff = od.date.signed_duration_since(query_date.unwrap()).abs().num_days();
            if diff > 1 {
                None
            } else {
                Some(od)
            }
        } else {
            None
        };
    if order.is_none() {
        info!("Query failed due to date mismatch.");
    }
    let template =
        OrderTemplate {
            orders,
            order_id: order_query.id,
            order_year: order_query.year,
            order_month: order_query.month,
            order_day: order_query.day,
            order};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "order.html")]
struct OrderTemplate {
    orders: glasgow_data::Orders,
    order_id: usize,
    order_year: i32,
    order_month: u32,
    order_day: u32,
    order: Option<glasgow_data::Order>
}

/// A wrapper type that we'll use to encapsulate HTML parsed by askama into valid HTML for axum to serve.
struct HtmlTemplate<T>(T);

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}