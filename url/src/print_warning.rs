use crate::url_warning::UrlWarning;

/// Prints a user-friendly message for each type of UrlWarning.
pub fn print_warning(warning: &UrlWarning) {
    match warning {
        UrlWarning::MissingScheme => {
            println!("Warning: The URL is missing a scheme (e.g., 'http', 'https').");
        }
        UrlWarning::InvalidPort(port) => {
            println!("Warning: The URL specifies an invalid port: '{}'.", port);
        }
        UrlWarning::InvalidHost(host) => {
            println!("Warning: The URL specifies an invalid host: '{}'.", host);
        }
        UrlWarning::MissingHost => {
            println!("Warning: The URL is missing a host (e.g., 'example.com').");
        }
        UrlWarning::UnknownScheme(scheme) => {
            println!("Warning: The URL uses an unknown scheme: '{}'.", scheme);
        }
    }
}
