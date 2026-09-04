use base_browser::cli::command::Command;
use browser_core::browser::browser::Browser;
use input_parser::traits::Parse;
use url::url::Url;
use utils::profile_scope;

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
        std::unimplemented!("Subcommand handling not yet implemented");
    } else if let Some(url_str) = command.opts.url {
        profile_scope!("Load Page");
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

        let mut browser = Browser::new();
        // browser.open_tab(url)?;
        browser.run();

        Ok(())
    } else {
        eprintln!(
            "Error: No URL or sub command was provided. Please specify a URL using a positional argument or with the '--url' option."
        );
        Err(anyhow::anyhow!("No URL or sub command was provided"))
    }
}
