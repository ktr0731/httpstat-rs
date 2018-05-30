extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate url;

use futures::{Future, Stream};
use hyper::Client;
use std::env;
use std::io::{self, Write};
use tokio_core::reactor::Core;
use url::{ParseError, Url};

const HTTPS_TEMPLATE: &str = "
  DNS Lookup   TCP Connection   TLS Handshake   Server Processing   Content Transfer
[%s  |     %s  |    %s  |        %s  |       %s  ]
|                |               |                   |                  |
namelookup:%s      |               |                   |                  |
connect:%s     |                   |                  |
pretransfer:%s         |                  |
starttransfer:%s        |
total:%s
";

const HTTP_TEMPLATE: &str = "
  DNS Lookup   TCP Connection   Server Processing   Content Transfer
[ %s  |     %s  |        %s  |       %s  ]
|                |                   |                  |
namelookup:%s      |                   |                  |
connect:%s         |                  |
starttransfer:%s        |
total:%s
";

fn main() {
    let args: Vec<String> = env::args().collect();

    request(&args[0]);
}

fn request(url: &str) -> Result<i32, String> {
    let mut core = Core::new()?;
    let client = Client::new(&core.handle());
    if let Ok(uri) = url.parse() {
        let work = client.get(uri).map(|res| {
            println!("Response: {}", res.status());
        });
    }
    Ok(0)
}
