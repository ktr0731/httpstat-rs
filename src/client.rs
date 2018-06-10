use serde_derive;
use serde_json;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::process;
use tempfile;

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
pub struct Metrics {
    pub time_namelookup: f32,
    pub time_connect: f32,
    pub time_pretransfer: f32,
    pub time_redirect: f32,
    pub time_starttransfer: f32,
    pub time_total: f32,
    pub speed_download: f32,
    pub speed_upload: f32,
    pub remote_ip: String,
    pub remote_port: String,
    pub local_ip: String,
    pub local_port: String,

    #[serde(skip_deserializing)]
    pub range_dns: f32,
    #[serde(skip_deserializing)]
    pub range_connection: f32,
    #[serde(skip_deserializing)]
    pub range_ssl: f32,
    #[serde(skip_deserializing)]
    pub range_server: f32,
    #[serde(skip_deserializing)]
    pub range_transfer: f32,
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

pub struct Headers {
    pub version: u8,
    pub code: u16,
    pub items: Vec<(String, String)>,
}

pub struct Body {
    pub filename: String,
    pub content: String,
}

pub struct Response {
    pub metrics: Metrics,
    pub headers: Headers,
    pub body: Body,
}

struct Tempfile {
    f: File,
    name: String,
}

impl Tempfile {
    fn new(filename: Option<String>) -> Result<Tempfile, String> {
        let file = tempfile::NamedTempFile::new()
            .map_err(|e| format!("failed to create temp file: {}", e))?;
        let filename = filename.unwrap_or(file.path().to_string_lossy().into_owned());

        let file = file.persist(&filename)
            .map_err(|e| format!("failed to persist temp file: {}", e))?;
        Ok(Tempfile {
            f: file,
            name: filename,
        })
    }

    fn reopen(&self) -> Result<Tempfile, String> {
        Ok(Tempfile {
            f: File::open(&self.name).map_err(|e| format!("failed to reopen header file: {}", e))?,
            name: self.name.clone(),
        })
    }

    fn path(&self) -> String {
        // TODO: Cow
        self.name.clone()
    }
}

impl Read for Tempfile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.f.read(buf)
    }
}

pub fn request(url: &str, body_filename: Option<String>) -> Result<Response, String> {
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

    let body_file = Tempfile::new(None)?;
    let header_file = Tempfile::new(None)?;

    let out = process::Command::new("curl")
        .args(&[
            "-w",
            CURL_FORMAT,
            "-D",
            &header_file.path(),
            "-o",
            &body_file.path(),
            "-s",
            "-S",
            url,
        ])
        .output()
        .map_err(|e| format!("failed to execute curl: {}", e))?
        .stdout;
    let resp = &String::from_utf8_lossy(&out);

    let header_file = header_file.reopen()?;

    let mut header_buf = BufReader::new(header_file);
    let mut headers = String::new();
    header_buf
        .read_to_string(&mut headers)
        .map_err(|e| format!("failed to read response header: {}", e))?;

    let mut lines = headers.trim().lines();

    // e.g. HTTP/1.1 200 -> ["HTTP1.1", "200"]
    let protocol_and_code: Vec<&str> = lines
        .next()
        .ok_or("expected protocol info, but EOF: {}")?
        .trim()
        .splitn(2, " ")
        .collect();
    let code: u16 = protocol_and_code[1]
        .parse()
        .map_err(|e| format!("failed to parse status code as interger: {}", e))?;
    let version: u8 = protocol_and_code[0]
        .split("/")
        .take(2)
        .collect::<Vec<&str>>()[1]
        .parse()
        .map_err(|e| format!("failed to parse protocol version as integer: {}", e))?;

    let mut header_items: Vec<(String, String)> = [].to_vec();
    for header in lines {
        let v: Vec<&str> = header.splitn(2, " ").collect();
        header_items.push((String::from(v[0]), String::from(v[1])));
    }

    let body_file = body_file.reopen()?;
    let body_filename = body_file.path().clone();

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
