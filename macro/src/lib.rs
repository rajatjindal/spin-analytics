use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn http_component_with_analytics(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;

    quote!(
        #[http_component]
        fn handle_http_request_mine(req: spin_http::Request) -> Result<spin_http::Response> {
            use spin_analytics::recorder::init_http_analytics;
            use spin_analytics::get_analytics_report;
            use anyhow::anyhow;

            let incoming_req = req.try_into().expect("cannot convert from Spin HTTP request");

            // the recording is done automatically inside `drop` function for `recorder`
            let mut recorder = init_http_analytics(&incoming_req);

            // called when /_analytics is called
            fn handle_http_analytics(_: Request) -> Result<Response>{
                Ok(http::Response::builder()
                .status(200)
                .body(Some(get_analytics_report().unwrap().into()))?)
            }

            #func

            let result = match incoming_req.uri().path() {
                "/_analytics" => handle_http_analytics(incoming_req),
                _ => #func_name(incoming_req),
            };

            let a = match result {
                Ok(resp) => {
                    let code: u16 = resp.status().as_u16();
                    recorder.set_http_status_code(code);
                    resp.try_into().expect("cannot convert to Spin HTTP response")
                },
                Err(e) => {
                    let body = e.to_string();
                       spin_http::Response {
                            status: 500,
                            headers: None,
                            body: Some(body.as_bytes().to_vec()),
                        }
                },
            };

            Ok(a)
        }
    )
    .into()
}
