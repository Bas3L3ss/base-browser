use std::process;

use base_browser::cli::command::Command;
use base_browser::parser::traits::Parse;
use base_browser::url::url::Url;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let command = match Command::parse(args) {
        Ok(parsed_output) => {
            for warning in &parsed_output.warnings {
                base_browser::cli::print_warning::print_warning(warning);
            }
            parsed_output.value
        }
        Err(err) => {
            eprintln!("Error: {err}");
            process::exit(1);
        }
    };

    if let Some(_sub_command) = command.sub_command {
        // handle sub_command
    } else if let Some(url_str) = command.opts.url {
        let parsed_url = match Url::parse(&url_str) {
            Ok(parsed_out) => {
                for warning in &parsed_out.warnings {
                    base_browser::url::print_warning::print_warning(warning);
                }
                parsed_out.value
            }
            Err(err) => {
                eprintln!("Error parsing URL: {err}");
                process::exit(1);
            }
        };

        println!("{parsed_url:?}");
    } else {
        eprintln!(
            "Error: No URL or sub command was provided. Please specify a URL using a positional argument or with the '--url' option."
        );
        process::exit(1);
    }
}
