use std::any::type_name_of_val;

use tracing::error;

pub trait TraceableError: std::fmt::Display {
    fn trace(&self) {
        let name: &str = type_name_of_val(self);
        let message: String = self.to_string();

        error!("exception.message" = message, "exception.type" = name);
    }
}

/// Convienence method to pass to map_err
pub fn trace_error<T>(error: T) -> T
where
    T: TraceableError + std::error::Error,
{
    error.trace();

    error
}
