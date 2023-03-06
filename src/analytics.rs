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
