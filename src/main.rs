use std::{println, process};

use base_browser::cli::{parse_cli::parse_cli, print_warning::print_warning};

fn main() {
    let parsed_output = match parse_cli() {
        Ok(mut parsed_output) => {
            // Pop warnings and print them until the list is empty
            while let Some(warning) = parsed_output.warnings.pop() {
                print_warning(warning);
            }
            parsed_output
        }

        Err(err) => {
            eprintln!("Error: {err}");
            process::exit(1);
        }
    };
}
