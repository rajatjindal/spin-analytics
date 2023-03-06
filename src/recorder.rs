use crate::consts;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use derive_builder::Builder;
use spin_sdk::http::Request;
use spin_sdk::key_value::Store;

/// Represents a Record
#[derive(Builder, Clone, Debug)]
#[builder(pattern = "owned")]
pub struct Record {
    #[builder(default)]
    pub trigger_type: String,

    #[builder(default)]
    pub component_id: String,

    #[builder(default)]
    pub path: String,

    #[builder(default)]
    pub execution_status: String,

    #[builder(default)]
    pub http_status_code: u16,

    #[builder(default)]
    pub http_method: String,

    #[builder(default)]
    pub start_time: DateTime<Utc>,

    #[builder(default = "Duration::zero()")]
    execution_time: Duration,
}

pub fn enable_http_analytics(req: &Request) -> Record {
    let x = RecordBuilder::default()
        .trigger_type("http".to_string())
        .http_method(req.method().to_string())
        .path(
            req.headers()
                .get("spin-path-info")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        )
        .start_time(Utc::now())
        .build()
        .unwrap();
    x
}

impl Record {
    pub fn set_http_status_code(&mut self, s: u16) -> &mut Record {
        self.http_status_code = s;
        self
    }

    fn set_execution_status(&mut self, s: String) -> &mut Record {
        self.execution_status = s;
        self
    }

    fn set_execution_time(&mut self, s: Duration) -> &mut Record {
        self.execution_time = s;
        self
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        self.set_execution_time(Utc::now() - self.start_time)
            .set_execution_status("ok".to_string());
        match collect_metric(self) {
            Ok(_) => {}
            Err(e) => println!("error when storing metric: {}", e),
        }
    }
}

pub fn collect_metric(record: &Record) -> Result<()> {
    let store_result = Store::open("default");
    let store = match store_result {
        Ok(store) => store,
        Err(err) => {
            return Err(err.into());
        }
    };

    increment_total_count(&store)?;
    increment_response_type(&store, &record)?;
    store_response_time_metric(&store, record)?;
    Ok(())
}

pub fn store_response_time_metric(store: &Store, record: &Record) -> Result<()> {
    let raw = store
        .get(consts::LAST_N_SUCCESS_RESPONSE_TIMES)
        .unwrap_or("[]".as_bytes().to_vec());
    let raw_str = std::str::from_utf8(&raw).unwrap();
    let mut response_times_ns_vec: Vec<i64> = serde_json::from_str(raw_str).unwrap();

    response_times_ns_vec.push(record.execution_time.num_microseconds().unwrap_or(0));
    let raw_str = serde_json::to_string(&response_times_ns_vec)?;

    store
        .set(consts::LAST_N_SUCCESS_RESPONSE_TIMES, raw_str)
        .map_err(anyhow::Error::msg)
}

pub fn store_p95(store: Store, response_times: Vec<i64>) -> Result<()> {
    let index = 95 / 100 * response_times.len();
    let x = response_times.get(index - 1).unwrap();
    store
        .set(consts::PERCENTILE_P95, format!("{}", x))
        .map_err(anyhow::Error::msg)
}

pub fn increment_response_type(store: &Store, record: &Record) -> Result<()> {
    match record.http_status_code {
        200 => increment_key(store, consts::TOTAL_SUCCESSFUL),
        401 => increment_key(store, consts::TOTAL_AUTH_N_ERROR),
        403 => increment_key(store, consts::TOTAL_AUTH_Z_ERROR),
        500 => increment_key(store, consts::TOTAL_SERVER_ERROR),
        _ => Ok(()),
    }
}

pub fn increment_total_count(store: &Store) -> Result<()> {
    increment_key(store, consts::TOTAL_REQUESTS)
}

pub fn increment_key(store: &Store, key: impl AsRef<str>) -> Result<()> {
    let value = store.get(&key).unwrap_or("0".as_bytes().to_vec());
    let count: i32 = std::str::from_utf8(&value).unwrap().parse().unwrap_or(0);
    store
        .set(key, format!("{}", count + 1))
        .map_err(anyhow::Error::msg)
}
