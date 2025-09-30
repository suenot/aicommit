use aicommit::logging::{LoggingConfig, init_logging, log_error, log_info, log_warning};
use aicommit::{trace, debug, info, warn, error};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct ExampleError {
    message: String,
}

impl fmt::Display for ExampleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Example error: {}", self.message)
    }
}

impl Error for ExampleError {}

fn main() -> anyhow::Result<()> {
    // Initialize logging with default configuration
    let mut config = LoggingConfig::new();
    config.with_debug(); // Enable debug logging for this example

    let _guard = init_logging(&config)?;

    // Test different log levels
    trace!("This is a trace message");
    debug!("This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");

    // Test structured logging with context
    log_info("Application started successfully", "main");
    log_warning("This is just a demonstration warning", "example");

    // Test error logging
    let example_error = ExampleError {
        message: "Something went wrong".to_string(),
    };

    let logged_error = log_error(example_error, "demonstration");
    println!("Error was logged and returned: {}", logged_error);

    // Test span-based tracing
    {
        let _span = aicommit::operation_span!("example_operation", user_id = "test123");
        info!("Working inside an operation span");
        debug!("This debug message will include span context");
    }

    info!("Logging example completed successfully");

    Ok(())
}