pub mod logging;

pub use logging::{
    LoggingConfig,
    init_logging,
    init_default_logging,
    init_dev_logging,
    init_prod_logging,
    log_error,
    log_info,
    log_warning,
};

// Re-export tracing macros for convenience
pub use tracing::{trace, debug, info, warn, error};