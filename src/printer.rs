use client;
use colored::{self, *};
use std::fmt;

// struct Printer<T: Write> {
pub struct Printer {
    // w: T,
    resp: client::Response,
}

impl Printer {
    pub fn new(resp: client::Response) -> Printer {
        Printer { resp: resp }
    }

    fn format_response_text(&self) -> String {
        let mut res = String::new();
        res.push_str(self.format_connection_text().as_str());
        res.push_str(self.format_header_text().as_str());
        res.push_str(self.format_body_location_text().as_str());
        res.push_str(self.format_body_text().as_str());
        res
    }

    fn format_connection_text(&self) -> String {
        let metrics = &self.resp.metrics;

        format!(
            "Connected to {}:{} from {}:{}\n\n",
            metrics.remote_ip.cyan(),
            metrics.remote_port.cyan(),
            metrics.local_ip,
            metrics.local_port
        )
    }

    fn format_header_text(&self) -> String {
        let headers = &self.resp.headers;
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

    fn format_body_location_text(&self) -> String {
        format!(
            "\n{} stored in: {}\n",
            "Body".green(),
            &self.resp.body.filename,
        )
    }

    fn format_body_text(&self) -> String {
        let metrics = &self.resp.metrics;

        if self.resp.headers.url.starts_with("https://") {
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
                a0000 = self.fmta(metrics.range_dns),
                a0001 = self.fmta(metrics.range_connection),
                a0002 = self.fmta(metrics.range_ssl),
                a0003 = self.fmta(metrics.range_server),
                a0004 = self.fmta(metrics.range_transfer),
                b0000 = self.fmtb(metrics.time_namelookup),
                b0001 = self.fmtb(metrics.time_connect),
                b0002 = self.fmtb(metrics.time_pretransfer),
                b0003 = self.fmtb(metrics.time_starttransfer),
                b0004 = self.fmtb(metrics.time_total),
            )
        } else {
            format!(
                "
  DNS Lookup   TCP Connection   Server Processing   Content Transfer
[   {a0000}  |     {a0001}    |      {a0002}      |      {a0003}    ]
|            |                |                   |                 |
    namelookup:{b0000}        |                   |                 |
                        connect:{b0001}           |                 |
                                      starttransfer:{b0002}         |
                                                                total:{b0003}
",
                a0000 = self.fmta(metrics.range_dns),
                a0001 = self.fmta(metrics.range_connection),
                a0002 = self.fmta(metrics.range_ssl),
                a0003 = self.fmta(metrics.range_server),
                b0000 = self.fmtb(metrics.time_namelookup),
                b0001 = self.fmtb(metrics.time_connect),
                b0002 = self.fmtb(metrics.time_pretransfer),
                b0003 = self.fmtb(metrics.time_starttransfer),
            )
        }
    }

    fn fmta(&self, n: f32) -> colored::ColoredString {
        format!("{:^7}", (n as i32).to_string() + "ms").cyan()
    }

    fn fmtb(&self, n: f32) -> colored::ColoredString {
        format!("{:<7}", (n as i32).to_string() + "ms").cyan()
    }
}

impl fmt::Display for Printer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format_response_text())
    }
}
