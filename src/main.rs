extern crate colored;
extern crate rand;
extern crate serde_json;
extern crate tempfile;
extern crate url;

#[macro_use]
extern crate serde_derive;

use rand::Rng;
use serde_json::{Error, Value};
use std::borrow::Cow;
use std::fs;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::{env, process};
use url::{ParseError, Url};

use colored::*;

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
struct Metrics {
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

impl Metrics {
    fn new(resp: &str) -> Result<Metrics, String> {
        let mut Metrics: Metrics = serde_json::from_str(resp)
            .map_err(|e| format!("failed to marshal response data: {}", e))?;
        Metrics.time_namelookup *= 1000.0;
        Metrics.time_connect *= 1000.0;
        Metrics.time_pretransfer *= 1000.0;
        Metrics.time_redirect *= 1000.0;
        Metrics.time_starttransfer *= 1000.0;
        Metrics.time_total *= 1000.0;

        Metrics.range_dns = Metrics.time_namelookup;
        Metrics.range_connection = Metrics.time_connect - Metrics.time_namelookup;
        Metrics.range_ssl = Metrics.time_pretransfer - Metrics.time_connect;
        Metrics.range_server = Metrics.time_starttransfer - Metrics.time_pretransfer;
        Metrics.range_transfer = Metrics.time_total - Metrics.time_starttransfer;
        let Metrics = Metrics;
        Ok(Metrics)
    }
}

struct Headers {
    version: u8,
    code: u16,
    items: Vec<(String, String)>,
}

struct Body {
    filename: String,
    content: String,
}

struct Response {
    metrics: Metrics,
    headers: Headers,
    body: Body,
}

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match env::args().nth(1) {
        Some(url) => {
            let res = formatResponseText(request(&url, None)?)?;
            println!("{}", res);
            Ok(())
        }
        None => Err(String::from("Usage: httpstat <url>")),
    }
}

fn createTempFile(filename: Option<String>) -> Result<(File, String), String> {
    let file =
        tempfile::NamedTempFile::new().map_err(|e| format!("failed to create temp file: {}", e))?;
    let filename = filename.unwrap_or(file.path().to_string_lossy().into_owned());

    let file = file.persist(&filename)
        .map_err(|e| format!("failed to persist temp file: {}", e))?;
    Ok((file, filename))
}

fn request(url: &str, body_filename: Option<String>) -> Result<Response, String> {
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
        }"#;

    let (body_file, body_filename) = createTempFile(None)?;
    let (header_file, header_filename) = createTempFile(None)?;

    let out = process::Command::new("curl")
        .args(&[
            "-w",
            CURL_FORMAT,
            "-D",
            &header_filename,
            "-o",
            &body_filename,
            "-s",
            "-S",
            url,
        ])
        .output()
        .map_err(|e| format!("failed to execute curl: {}", e))?
        .stdout;
    let resp = &String::from_utf8_lossy(&out);

    let header_file =
        File::open(header_filename).map_err(|e| format!("failed to reopen header file: {}", e))?;
    let mut header_buf = BufReader::new(header_file);
    let mut headers = String::new();
    header_buf
        .read_to_string(&mut headers)
        .map_err(|e| format!("failed to read response header: {}", e))?;

    let mut lines = headers.trim().lines();
    let protocol_and_code: Vec<&str> = lines
        .next()
        .ok_or("expected protocol info, but EOF: {}")?
        .trim()
        .splitn(2, " ")
        .collect();
    let code: u16 = protocol_and_code[1]
        .parse()
        .map_err(|e| format!("failed to parse status code as interger: {}", e))?;
    let protocol_and_version: Vec<&str> = protocol_and_code[0].split("/").take(2).collect();
    let version: u8 = protocol_and_version[1]
        .parse()
        .map_err(|e| format!("failed to parse protocol version as integer: {}", e))?;

    let mut header_items: Vec<(String, String)> = [].to_vec();
    for header in lines {
        let v: Vec<&str> = header.splitn(2, " ").collect();
        header_items.push((String::from(v[0]), String::from(v[1])));
    }

    let body_file =
        File::open(&body_filename).map_err(|e| format!("failed to reopen body file: {}", e))?;
    let mut body_buf = BufReader::new(body_file);
    let mut body = String::new();
    body_buf
        .read_to_string(&mut body)
        .map_err(|e| format!("failed to read response body: {}", e))?;

    Ok(Response {
        headers: Headers {
            version: version,
            code: code,
            items: header_items,
        },
        body: Body {
            filename: body_filename,
            content: body,
        },
        metrics: Metrics::new(resp)?,
    })
}

fn formatResponseText(resp: Response) -> Result<String, String> {
    let mut res = String::new();
    res.push_str(formatConnection(&resp.metrics).as_str());
    res.push_str(formatHeaders(&resp.headers).as_str());
    res.push_str(formatBodyLocation(&resp.body.filename).as_str());
    res.push_str(formatBody(&resp.metrics).as_str());
    Ok(res)
}

fn formatConnection(Metrics: &Metrics) -> String {
    format!(
        "Connected to {}:{} from {}:{}\n\n",
        Metrics.remote_ip.cyan(),
        Metrics.remote_port.cyan(),
        Metrics.local_ip,
        Metrics.local_port
    )
}

fn formatHeaders(headers: &Headers) -> String {
    let mut s = String::new();
    s.push_str(
        format!(
            "{}/{} {}\n",
            "HTTP".green(),
            headers.version.to_string().cyan(),
            headers.code.to_string().cyan()
        ).as_str(),
    );
    for header in &headers.items {
        s.push_str(format!("{} {}\n", header.0, header.1.cyan(),).as_str())
    }
    s
}

fn formatBodyLocation(loc: &str) -> String {
    format!("\n{} stored in: {}\n", "Body".green(), loc)
}

fn formatBody(Metrics: &Metrics) -> String {
    format!(
        "
  DNS Lookup   TCP Connection   TLS Handshake   Server Processing   Content Transfer
[   {a0000}  |     {a0001}    |    {a0002}    |      {a0003}      |      {a0004}     ]
             |                |               |                   |                  |
    namelookup:{b0000}        |               |                   |                  |
                        connect:{b0001}       |                   |                  |
                                    pretransfer:{b0002}           |                  |
                                                      starttransfer:{b0003}          |
                                                                                 total:{b0004}

",
        a0000 = fmta(Metrics.range_dns),
        a0001 = fmta(Metrics.range_connection),
        a0002 = fmta(Metrics.range_ssl),
        a0003 = fmta(Metrics.range_server),
        a0004 = fmta(Metrics.range_transfer),
        b0000 = fmtb(Metrics.time_namelookup),
        b0001 = fmtb(Metrics.time_connect),
        b0002 = fmtb(Metrics.time_pretransfer),
        b0003 = fmtb(Metrics.time_starttransfer),
        b0004 = fmtb(Metrics.time_total),
    )
}

fn fmta(n: f32) -> colored::ColoredString {
    format!("{:^7}", (n as i32).to_string() + "ms").cyan()
}

fn fmtb(n: f32) -> colored::ColoredString {
    format!("{:<7}", (n as i32).to_string() + "ms").cyan()
}

fn get_random_filename() -> String {
    rand::thread_rng()
        .gen_ascii_chars()
        .take(10)
        .collect::<String>()
}
