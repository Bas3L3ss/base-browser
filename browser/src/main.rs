use base_browser::cli::command::Command;
use html::{parser::parser::Parser, print_dom_tree};
use input_parser::traits::Parse;
use network::http::client::HttpClient;
use url::url::Url;

fn main() -> anyhow::Result<()> {
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
            return Err(err.into());
        }
    };

    if let Some(_sub_command) = command.sub_command {
        // handle sub_command
        return Ok(());
    } else if let Some(url_str) = command.opts.url {
        let url = match Url::parse(&url_str) {
            Ok(parsed_out) => {
                for warning in &parsed_out.warnings {
                    url::print_warning::print_warning(warning);
                }
                parsed_out.value
            }
            Err(err) => {
                eprintln!("Error parsing URL: {err}");
                return Err(err.into());
            }
        };

        let http = HttpClient::new();
        let response = http.get(&url)?;
        let html_parse_result = Parser::to_dom(&response.body_as_string());

        print_dom_tree(&html_parse_result.arena);

        Ok(())
    } else {
        eprintln!(
            "Error: No URL or sub command was provided. Please specify a URL using a positional argument or with the '--url' option."
        );
        Err(anyhow::anyhow!("No URL or sub command was provided"))
    }
}
