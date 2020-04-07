mod clap_handler;
mod util;

use clap_handler::app::request_from_args;

fn main() {
    match request_from_args() {
        Ok(o) => println!("Successfully parsed command line arguments.\n{:?}", o),
        Err(e) => println!("{}", e),
    }
}
