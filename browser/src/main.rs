use base_browser::cli::command::Command;
use html::dom::{node_type::NodeType, stylesheet::StyleSheet};
use input_parser::traits::Parse;
use network::http::client::HttpClient;
use std::unimplemented;
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
        unimplemented!("Subcommand handling not yet implemented");
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

        let http = HttpClient::new();
        let response = http.get(&url)?;
        let dom = html::parser::parser::Parser::to_dom(&response.body_as_string())?;

        for style_sheet in dom.value.style_sheets {
            match style_sheet {
                StyleSheet::InlineStyle(node_id) => {
                    let children_id = dom.value.nodes[node_id].children[0];
                    let css = match &dom.value.nodes[children_id].ntype {
                        NodeType::Text(content) => content,
                        _ => continue,
                    };

                    //
                }
                StyleSheet::StyleSheetLink(node_id) => {
                    // Try to extract href from the element node
                    let href = match &dom.value.nodes[node_id].ntype {
                        NodeType::Element(element) => match &element.attributes.href {
                            Some(href) => href,
                            None => continue,
                        },
                        _ => continue,
                    };

                    let css_url = url.resolve(href)?;
                    let css_response = http.get(&css_url)?;
                }
            }
        }

        Ok(())
    } else {
        eprintln!(
            "Error: No URL or sub command was provided. Please specify a URL using a positional argument or with the '--url' option."
        );
        Err(anyhow::anyhow!("No URL or sub command was provided"))
    }
}
