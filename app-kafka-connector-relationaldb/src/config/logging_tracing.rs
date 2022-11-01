use common_error::AppError;
use common_tracing::initialize_logging_and_tracing;

use crate::config::configuration::Configuration;

pub fn init(config: &Configuration) -> Result<(), AppError> {
    Ok(initialize_logging_and_tracing(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        |mut e| {
            for directive in &config.logging.level.directives {
                let directive_string = format!("{}={}", directive.namespace, directive.level);
                e = e.add_directive(directive_string.parse().unwrap_or_default());
            }
            e
        },
    )?)
}
