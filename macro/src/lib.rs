use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn http_component_with_analytics(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;

    quote!(
        #[http_component]
        fn handle_http_request_mine(req: spin_http::Request) -> Result<spin_http::Response> {
            use spin_analytics::recorder::enable_http_analytics;
            use spin_analytics::get_html;
            use anyhow::anyhow;
            let xy = req.try_into().expect("cannot convert from Spin HTTP request");
            let mut recorder = enable_http_analytics(&xy);
            fn handle_http_analytics(_: Request) -> Result<Response>{
                Ok(http::Response::builder()
                .status(200)
                .body(Some(get_html().unwrap().into()))?)
            }

            #func

            let result = match xy.uri().path() {
                "/_analytics" => handle_http_analytics(xy),
                _ => #func_name(xy),
            };
            // let result = #func_name(xy);
            let a = match result {
                Ok(resp) => {
                    let code: u16 = resp.status().as_u16();
                    recorder.set_http_status_code(code);
                    resp.try_into().expect("cannot convert to Spin HTTP response")
                },
                Err(e) => {
                    println!("from inside error");
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
