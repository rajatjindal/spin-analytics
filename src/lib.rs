pub mod analytics;
pub mod consts;
pub mod recorder;
use anyhow::Result;
pub use spin_analytics_macro::*;
use spin_sdk::key_value::Store;

pub fn get_html() -> Result<String> {
    let store_result = Store::open("default");
    let store = match store_result {
        Ok(store) => store,
        Err(err) => return Err(err.into()),
    };

    let data = analytics::get_analytics_data(&store)?;
    let html = format!(
        r#"
<!doctype html>
<html>

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <script src="https://cdn.tailwindcss.com"></script>
</head>

<body>
    <div class="grid grid-cols-5 gap-4 mt-28 mx-auto w-10/12">
        <div class="col-span-1 border h-40">
            <div class="h-1/4 text-center py-2 font-bold text-sm border-b">
                Total Requests
            </div>
            <div class="h-3/4 py-2 font-bold my-auto flex items-center justify-center">
                {}
            </div>
        </div>

        <div class="col-span-1 border h-40">
            <div class="h-1/4 text-center py-2 font-bold text-sm border-b">
                Total Successful
            </div>
            <div class="h-3/4 py-2 font-bold my-auto flex items-center justify-center">
                {}
            </div>
        </div>

        <div class="col-span-1 border h-40">
            <div class="h-1/4 text-center py-2 font-bold text-sm border-b">
                Total Unauthorized
            </div>
            <div class="h-3/4 py-2 font-bold my-auto flex items-center justify-center">
                {}
            </div>
        </div>

        <div class="col-span-1 border h-40">
            <div class="h-1/4 text-center py-2 font-bold text-sm border-b">
                Total Server error
            </div>
            <div class="h-3/4 py-2 font-bold my-auto flex items-center justify-center">
                {}
            </div>
        </div>
    </div>
</body>

</html>
"#,
        data.total_requests,
        data.total_successful,
        data.total_auth_n_error + data.total_auth_z_error,
        data.total_auth_z_error
    );

    Ok(html)
}
