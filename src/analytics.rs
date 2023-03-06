use crate::consts;
use anyhow::Result;
use spin_sdk::key_value::Store;

pub fn get_all_data(max_records: i32) -> Result<Vec<Vec<u8>>> {
    let store_result = Store::open("default");
    let store = match store_result {
        Ok(store) => store,
        Err(err) => return Err(err.into()),
    };

    let keys = match store.get_keys() {
        Ok(keys) => keys,
        Err(err) => panic!("{:?}", err),
    };

    let mut values = vec![];
    let mut index = 0;
    for key in keys {
        if index >= max_records {
            break;
        }

        let value = match store.get(key) {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };

        values.push(value);
        index = index + 1;
    }

    Ok(values)
}

pub struct Analytics {
    pub total_requests: i64,
    pub total_successful: i64,
    pub total_auth_n_error: i64,
    pub total_auth_z_error: i64,
    pub total_server_error: i64,
    pub last_n_response_times: Vec<i64>,
}

// getters
pub fn get_analytics_data(store: &Store) -> Result<Analytics> {
    Ok(Analytics {
        total_requests: get_i64(store, consts::TOTAL_REQUESTS).unwrap(),
        total_successful: get_i64(store, consts::TOTAL_SUCCESSFUL).unwrap(),
        total_auth_n_error: get_i64(store, consts::TOTAL_AUTH_N_ERROR).unwrap(),
        total_auth_z_error: get_i64(store, consts::TOTAL_AUTH_Z_ERROR).unwrap(),
        total_server_error: get_i64(store, consts::TOTAL_SERVER_ERROR).unwrap(),
        last_n_response_times: get_response_time_metric(store).unwrap(),
    })
}

pub fn get_p95(store: &Store) -> Result<i64> {
    get_i64(store, consts::PERCENTILE_P95)
}

pub fn get_i64(store: &Store, key: impl AsRef<str>) -> Result<i64> {
    let raw = store.get(key).unwrap_or_default();
    let raw_str = std::str::from_utf8(&raw).unwrap_or("-1");
    if raw_str == "" {
        return Ok(0);
    }

    raw_str.parse().map_err(anyhow::Error::msg)
}

pub fn get_response_time_metric(store: &Store) -> Result<Vec<i64>> {
    let raw = match store.get(consts::LAST_N_SUCCESS_RESPONSE_TIMES) {
        Ok(raw) => raw,
        Err(_) => "[]".as_bytes().to_vec(),
    };

    let raw_str = std::str::from_utf8(&raw).unwrap_or("[]");
    serde_json::from_str(raw_str).map_err(anyhow::Error::msg)
}
