This repository contains code to analyze fulfillment data for the [Glasgow - Digital Interface Explorer](http://glasgow-embedded.org) campaign on [CrowdSupply](https://www.crowdsupply.com/1bitsquared/glasgow).

The project consists of three parts:
* Library containing all the data parsing and calculation logic.
* A cli (Command Line Interface) to allow quick lookup of stats and order numbers. (meant for project maintainers)
* A web server. This part is meant to be deployed as a website, to allow backers self service lookup of their order status.

The whole project is written in Rust. Why? Because I wanted to see if it is useful for a thing that usually is done with a pile of Python scripts. Turns out Rust is a lot of fun and a great tool for this!

## License

This project, as it is typical for Rust projects is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Code of Conduct

Please refer to the [Glasgow project Code of Conduct](http://glasgow-embedded.org/latest/conduct.html)
