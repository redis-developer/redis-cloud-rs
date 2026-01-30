//! Python bindings for Redis Cloud API client

use crate::error::IntoPyResult;
use crate::runtime::{block_on, future_into_py};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use redis_cloud::{CloudClient, DatabaseHandler, SubscriptionHandler};
use std::sync::Arc;
use std::time::Duration;

/// Redis Cloud API client
#[pyclass(name = "CloudClient")]
pub struct PyCloudClient {
    client: Arc<CloudClient>,
}

#[pymethods]
impl PyCloudClient {
    /// Create a new Redis Cloud client
    #[new]
    #[pyo3(signature = (api_key, api_secret, base_url=None, timeout_secs=None))]
    fn new(
        api_key: String,
        api_secret: String,
        base_url: Option<String>,
        timeout_secs: Option<u64>,
    ) -> PyResult<Self> {
        let mut builder = CloudClient::builder()
            .api_key(api_key)
            .api_secret(api_secret);

        if let Some(url) = base_url {
            builder = builder.base_url(url);
        }

        if let Some(secs) = timeout_secs {
            builder = builder.timeout(Duration::from_secs(secs));
        }

        let client = builder.build().into_py_result()?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Create client from environment variables
    #[staticmethod]
    fn from_env() -> PyResult<Self> {
        let api_key = std::env::var("REDIS_CLOUD_API_KEY")
            .or_else(|_| std::env::var("REDIS_CLOUD_ACCOUNT_KEY"))
            .map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(
                    "API key not found. Set REDIS_CLOUD_API_KEY or REDIS_CLOUD_ACCOUNT_KEY",
                )
            })?;

        let api_secret = std::env::var("REDIS_CLOUD_API_SECRET")
            .or_else(|_| std::env::var("REDIS_CLOUD_SECRET_KEY"))
            .or_else(|_| std::env::var("REDIS_CLOUD_USER_KEY"))
            .map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(
                    "API secret not found. Set REDIS_CLOUD_API_SECRET",
                )
            })?;

        let mut builder = CloudClient::builder()
            .api_key(api_key)
            .api_secret(api_secret);

        if let Ok(base_url) = std::env::var("REDIS_CLOUD_BASE_URL") {
            builder = builder.base_url(base_url);
        }

        let client = builder.build().into_py_result()?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    // Subscriptions API

    /// List all subscriptions (async)
    fn subscriptions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = SubscriptionHandler::new((*client).clone());
            let subs = handler.get_all_subscriptions().await.into_py_result()?;
            let json = serde_json::to_value(&subs)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// List all subscriptions (sync)
    fn subscriptions_sync(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = SubscriptionHandler::new((*client).clone());
            handler.get_all_subscriptions().await.into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    /// Get a specific subscription by ID (async)
    fn subscription<'py>(
        &self,
        py: Python<'py>,
        subscription_id: i64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = SubscriptionHandler::new((*client).clone());
            let sub = handler
                .get_subscription_by_id(subscription_id as i32)
                .await
                .into_py_result()?;
            let json = serde_json::to_value(&sub)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// Get a specific subscription by ID (sync)
    fn subscription_sync(&self, py: Python<'_>, subscription_id: i64) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = SubscriptionHandler::new((*client).clone());
            handler
                .get_subscription_by_id(subscription_id as i32)
                .await
                .into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    // Databases API

    /// List databases in a subscription (async)
    #[pyo3(signature = (subscription_id, offset=None, limit=None))]
    fn databases<'py>(
        &self,
        py: Python<'py>,
        subscription_id: i64,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = DatabaseHandler::new((*client).clone());
            let dbs = handler
                .get_subscription_databases(subscription_id as i32, offset, limit)
                .await
                .into_py_result()?;
            let json = serde_json::to_value(&dbs)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// List databases in a subscription (sync)
    #[pyo3(signature = (subscription_id, offset=None, limit=None))]
    fn databases_sync(
        &self,
        py: Python<'_>,
        subscription_id: i64,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = DatabaseHandler::new((*client).clone());
            handler
                .get_subscription_databases(subscription_id as i32, offset, limit)
                .await
                .into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    /// Get a specific database by ID (async)
    fn database<'py>(
        &self,
        py: Python<'py>,
        subscription_id: i64,
        database_id: i64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = DatabaseHandler::new((*client).clone());
            let db = handler
                .get_subscription_database_by_id(subscription_id as i32, database_id as i32)
                .await
                .into_py_result()?;
            let json = serde_json::to_value(&db)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// Get a specific database by ID (sync)
    fn database_sync(
        &self,
        py: Python<'_>,
        subscription_id: i64,
        database_id: i64,
    ) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = DatabaseHandler::new((*client).clone());
            handler
                .get_subscription_database_by_id(subscription_id as i32, database_id as i32)
                .await
                .into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    // Raw API access

    /// Execute a raw GET request (async)
    fn get<'py>(&self, py: Python<'py>, path: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let result = client.get_raw(&path).await.into_py_result()?;
            Python::with_gil(|py| Ok(json_to_py(py, result)))
        })
    }

    /// Execute a raw GET request (sync)
    fn get_sync(&self, py: Python<'_>, path: String) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(
            py,
            async move { client.get_raw(&path).await.into_py_result() },
        )?;
        Ok(json_to_py(py, result))
    }

    /// Execute a raw POST request (async)
    fn post<'py>(
        &self,
        py: Python<'py>,
        path: String,
        body: Py<PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let body_json = py_to_json(py, body)?;
        let client = self.client.clone();
        future_into_py(py, async move {
            let result = client.post_raw(&path, body_json).await.into_py_result()?;
            Python::with_gil(|py| Ok(json_to_py(py, result)))
        })
    }

    /// Execute a raw POST request (sync)
    fn post_sync(&self, py: Python<'_>, path: String, body: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let body_json = py_to_json(py, body)?;
        let client = self.client.clone();
        let result = block_on(py, async move {
            client.post_raw(&path, body_json).await.into_py_result()
        })?;
        Ok(json_to_py(py, result))
    }

    /// Execute a raw DELETE request (async)
    fn delete<'py>(&self, py: Python<'py>, path: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let result = client.delete_raw(&path).await.into_py_result()?;
            Python::with_gil(|py| Ok(json_to_py(py, result)))
        })
    }

    /// Execute a raw DELETE request (sync)
    fn delete_sync(&self, py: Python<'_>, path: String) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            client.delete_raw(&path).await.into_py_result()
        })?;
        Ok(json_to_py(py, result))
    }
}

/// Convert serde_json::Value to Python object
pub fn json_to_py(py: Python<'_>, value: serde_json::Value) -> Py<PyAny> {
    match value {
        serde_json::Value::Null => py.None(),
        serde_json::Value::Bool(b) => b.into_pyobject(py).unwrap().to_owned().into_any().unbind(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_pyobject(py).unwrap().to_owned().into_any().unbind()
            } else if let Some(f) = n.as_f64() {
                f.into_pyobject(py).unwrap().to_owned().into_any().unbind()
            } else {
                py.None()
            }
        }
        serde_json::Value::String(s) => s.into_pyobject(py).unwrap().to_owned().into_any().unbind(),
        serde_json::Value::Array(arr) => {
            let list = PyList::new(py, arr.into_iter().map(|v| json_to_py(py, v))).unwrap();
            list.into_any().unbind()
        }
        serde_json::Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (k, v) in obj {
                dict.set_item(k, json_to_py(py, v)).unwrap();
            }
            dict.into_any().unbind()
        }
    }
}

/// Convert Python object to serde_json::Value
pub fn py_to_json(py: Python<'_>, obj: Py<PyAny>) -> PyResult<serde_json::Value> {
    let obj = obj.bind(py);

    if obj.is_none() {
        Ok(serde_json::Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(serde_json::Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(serde_json::Value::Number(i.into()))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(serde_json::json!(f))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(serde_json::Value::String(s))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let arr: PyResult<Vec<serde_json::Value>> = list
            .iter()
            .map(|item| py_to_json(py, item.unbind()))
            .collect();
        Ok(serde_json::Value::Array(arr?))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            map.insert(key, py_to_json(py, v.unbind())?);
        }
        Ok(serde_json::Value::Object(map))
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Cannot convert Python object to JSON",
        ))
    }
}
