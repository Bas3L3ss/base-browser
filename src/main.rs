use std::{println, process};

use base_browser::{
    cli::{parse_cli::parse_cli, print_warning::print_warning},
    navigation::url::Url,
};

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

    if let Some(_sub_command) = parsed_output.sub_command {
        // handle sub_command
    } else if let Some(url_str) = parsed_output.opts.url {
        let parsed_url = Url::parse(&url_str);
        println!("{parsed_url:?}");
    } else {
        eprintln!(
            "Error: No URL or sub command was provided. Please specify a URL using a positional argument or with the '--url' option."
        );
        process::exit(1);
    }
}
