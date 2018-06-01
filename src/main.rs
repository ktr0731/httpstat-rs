extern crate colored;
extern crate serde_json;
extern crate tempfile;
extern crate url;

#[macro_use]
extern crate serde_derive;

use serde_json::{Error, Value};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::{env, process};
use url::{ParseError, Url};

use colored::*;

const CURL_FORMAT: &str = r#"
{
    "time_namelookup":    %{time_namelookup},
    "time_connect":       %{time_connect},
    "time_pretransfer":   %{time_pretransfer},
    "time_redirect":      %{time_redirect},
    "time_starttransfer": %{time_starttransfer},
    "time_total":         %{time_total},
    "speed_download":     %{speed_download},
    "speed_upload":       %{speed_upload},
    "remote_ip":          "%{remote_ip}",
    "remote_port":        "%{remote_port}",
    "local_ip":           "%{local_ip}",
    "local_port":         "%{local_port}"
}
"#;

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
//

#[derive(Serialize, Deserialize, Debug)]
struct Status {
    time_namelookup: f32,
    time_connect: f32,
    time_pretransfer: f32,
    time_redirect: f32,
    time_starttransfer: f32,
    time_total: f32,
    speed_download: f32,
    speed_upload: f32,
    remote_ip: String,
    remote_port: String,
    local_ip: String,
    local_port: String,

    #[serde(skip_deserializing)]
    range_dns: f32,
    #[serde(skip_deserializing)]
    range_connection: f32,
    #[serde(skip_deserializing)]
    range_ssl: f32,
    #[serde(skip_deserializing)]
    range_server: f32,
    #[serde(skip_deserializing)]
    range_transfer: f32,
}

impl Status {
    fn new(resp: &str) -> Result<Status, String> {
        let mut status: Status = serde_json::from_str(resp)
            .map_err(|e| format!("failed to marshal response data: {}", e))?;
        status.time_namelookup *= 1000.0;
        status.time_connect *= 1000.0;
        status.time_pretransfer *= 1000.0;
        status.time_redirect *= 1000.0;
        status.time_starttransfer *= 1000.0;
        status.time_total *= 1000.0;

        status.range_dns = status.time_namelookup;
        status.range_connection = status.time_connect - status.time_namelookup;
        status.range_ssl = status.time_pretransfer - status.time_connect;
        status.range_server = status.time_starttransfer - status.time_pretransfer;
        status.range_transfer = status.time_total - status.time_starttransfer;
        let status = status;
        Ok(status)
    }
}

fn main() {
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
            let res = formatResponseText(request(&url)?)?;
            println!("{}", res);
            Ok(())
        }
        None => Err(String::from("Usage: httpstat <url>")),
    }
}

fn request(url: &str) -> Result<Status, String> {
    let out = process::Command::new("curl")
        .args(&[
            "-w",
            CURL_FORMAT,
            "-D",
            "tmpd",
            "-o",
            "tmpo",
            "-s",
            "-S",
            url,
        ])
        .output()
        .map_err(|e| format!("failed to execute curl: {}", e))?
        .stdout;
    let resp = &String::from_utf8_lossy(&out);
    Ok(Status::new(resp)?)
}

fn formatResponseText(status: Status) -> Result<String, String> {
    Ok(format!(
        "
  DNS Lookup   TCP Connection   TLS Handshake   Server Processing   Content Transfer
[{}  |     {}  |    {}  |        {}  |       {}  ]
            |                |               |                   |                  |
   namelookup:{}        |               |                   |                  |
                       connect:{}       |                   |                  |
                                   pretransfer:{}           |                  |
                                                     starttransfer:{}          |
                                                                                total:{}

",
        fmta(status.range_dns),
        fmta(status.range_connection),
        fmta(status.range_ssl),
        fmta(status.range_server),
        fmta(status.range_transfer),
        fmtb(status.time_namelookup),
        fmtb(status.time_connect),
        fmtb(status.time_pretransfer),
        fmtb(status.time_starttransfer),
        fmtb(status.time_total),
    ))
}

fn fmta(n: f32) -> colored::ColoredString {
    format!("{:7}ms", n as i32).cyan()
}

fn fmtb(n: f32) -> colored::ColoredString {
    format!("{:<7}", (n as i32).to_string() + "ms").cyan()
}
