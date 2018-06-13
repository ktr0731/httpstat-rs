extern crate getopts;
extern crate httpstat;

use getopts::Options;
use httpstat::{client, printer};
use std::{env, process};

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut opts = Options::new();
    opts.optflag("h", "help", "show this screen.");

    let args: Vec<String> = env::args().collect();

    let matches = opts.parse(&args[1..])
        .map_err(|e| format!("failed to parse options: {}", e))?;

    // if matches.opt_present("")

    match matches.free.get(1) {
        Some(url) => {
            let resp = client::request(&url, None)?;
            let printer = printer::Printer::new(resp);
            println!("{}", printer);
            Ok(())
        }
        None => {
            print_usage();
            Ok(())
        }
    }
}

fn print_usage() {
    println!(
        "Usage: httpstat URL [CURL_OPTIONS]
       httpstat -h | --help
       httpstat --version

Arguments:
  URL     url to request, could be with or without `http(s)://` prefix

Options:
  CURL_OPTIONS  any curl supported options, except for -w -D -o -S -s,
                which are already used internally.
  -h --help     show this screen.
  --version     show version.

Environments:
  HTTPSTAT_SHOW_BODY    Set to `true` to show response body in the output,
                        note that body length is limited to 1023 bytes, will be
                        truncated if exceeds. Default is `false`.
  HTTPSTAT_SHOW_IP      By default httpstat shows remote and local IP/port address.
                        Set to `false` to disable this feature. Default is `true`.
  HTTPSTAT_SHOW_SPEED   Set to `true` to show download and upload speed.
                        Default is `false`.
  HTTPSTAT_SAVE_BODY    By default httpstat stores body in a tmp file,
                        set to `false` to disable this feature. Default is `true`
  HTTPSTAT_CURL_BIN     Indicate the curl bin path to use. Default is `curl`
                        from current shell $PATH.
  HTTPSTAT_DEBUG        Set to `true` to see debugging logs. Default is `false`
"
    )
}
