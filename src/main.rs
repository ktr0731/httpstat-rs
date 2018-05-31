extern crate reqwest;
extern crate url;

use reqwest::Error;
use std::io::{self, Write};
use std::{env, process};
use url::{ParseError, Url};

// const HTTPS_TEMPLATE: &str = "
//   DNS Lookup   TCP Connection   TLS Handshake   Server Processing   Content Transfer
// [%s  |     %s  |    %s  |        %s  |       %s  ]
// |                |               |                   |                  |
// namelookup:%s      |               |                   |                  |
// connect:%s     |                   |                  |
// pretransfer:%s         |                  |
// starttransfer:%s        |
// total:%s
// ";
//
// const HTTP_TEMPLATE: &str = "
//   DNS Lookup   TCP Connection   Server Processing   Content Transfer
// [ %s  |     %s  |        %s  |       %s  ]
// |                |                   |                  |
// namelookup:%s      |                   |                  |
// connect:%s         |                  |
// starttransfer:%s        |
// total:%s
// ";

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{}", &args[1]);
    process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            println!("{}", err);
            1
        }
    })
}

fn run() -> Result<(), String> {
    match env::args().nth(1) {
        Some(url) => {
            let res = request(&url)?;
            println!("{}", res);
            Ok(())
        }
        None => Err(String::from("Usage: httpstat <url>")),
    }
}

fn request(url: &str) -> Result<String, String> {
    reqwest::get(url)
        .map_err(|e| format!("failed to send request: {}", e))?
        .text()
        .map_err(|e| format!("failed: {}", e))
    // .map(|res| {
    //     println!("Response: {}", res.status());
    // });
    // Ok(0)
}
