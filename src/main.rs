mod clap_handler;
mod gpu72_runtime;
mod primenet_runtime;
mod util;

use clap_handler::app::{request_from_args, Options};
use gpu72_runtime::{gpu72_cleanup, gpu72_runtime};
use primenet_runtime::{primenet_cleanup, primenet_runtime};

fn main() {
    match request_from_args() {
        Ok(o) => {
            println!("Successfully parsed command line arguments.");
            match (o, o.clone()) {
                (Options::Primenet(primenet_options), Options::Primenet(clone)) => {
                    if primenet_runtime(primenet_options).is_err() {
                        primenet_cleanup(clone);
                    }
                }
                (Options::Gpu72(gpu72_options), Options::Gpu72(clone)) => {
                    if gpu72_runtime(gpu72_options).is_err() {
                        // gpu72_cleanup(clone);
                        println!("oh no");
                    }
                }
                _ => unreachable!(),
            }
        }
        Err(e) => println!("{}", e),
    }
}
