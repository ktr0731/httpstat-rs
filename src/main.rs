extern crate httpstat;

use httpstat::{client, printer};

use std::{env, process};

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match env::args().nth(1) {
        Some(url) => {
            let resp = client::request(&url, None)?;
            let printer = printer::Printer::new(resp);
            println!("{}", printer);
            Ok(())
        }
        None => Err(String::from("Usage: httpstat <url>")),
    }
}
