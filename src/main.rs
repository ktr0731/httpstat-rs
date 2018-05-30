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

const HTTP_TEMPLATE: &str = " \
                             DNS Lookup   TCP Connection   Server Processing   Content Transfer \
                             [ %s  |     %s  |        %s  |       %s  ] \
                             |                |                   |                  | \
                             namelookup:%s      |                   |                  | \
                             connect:%s         |                  | \
                             starttransfer:%s        | \
                             total:%s \
                             ";

fn main() {
    println!("{}", HTTPS_TEMPLATE);
}
