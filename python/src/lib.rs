//! Python bindings for Redis Cloud API client
//!
//! Provides both async and sync APIs for managing Redis Cloud resources.

use pyo3::prelude::*;

mod client;
mod error;
mod runtime;

use client::PyCloudClient;
use error::RedisCloudError;

/// Python module for Redis Cloud API client
#[pymodule]
fn redis_cloud(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("RedisCloudError", m.py().get_type::<RedisCloudError>())?;
    m.add_class::<PyCloudClient>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
