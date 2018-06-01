extern crate url;

use std::io::{self, Write};
use std::{env, process};
use url::{ParseError, Url};

const CURL_FORMAT: &str = "
{
    content_type:       %{content_type},
    filename_effective: %{filename_effective},
    ftp_entry_path:     %{ftp_entry_path},
    http_code:          %{http_code},
    http_connect:       %{http_connect},
    local_ip:           %{local_ip},
    local_port:         %{local_port},
    num_connects:       %{num_connects},
    num_redirects:      %{num_redirects},
    redirect_url:       %{redirect_url},
    remote_ip:          %{remote_ip},
    remote_port:        %{remote_port},
    size_download:      %{size_download},
    size_header:        %{size_header},
    size_request:       %{size_request},
    size_upload:        %{size_upload},
    speed_download:     %{speed_download},
    speed_upload:       %{speed_upload},
    ssl_verify_result:  %{ssl_verify_result},
    time_appconnect:    %{time_appconnect},
    time_connect:       %{time_connect},
    time_namelookup:    %{time_namelookup},
    time_pretransfer:   %{time_pretransfer},
    time_redirect:      %{time_redirect},
    time_starttransfer: %{time_starttransfer},
    time_total:         %{time_total},
    url_effective:      %{url_effective}
}
";

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
    Ok(String::from_utf8_lossy(&out).to_string())
}
