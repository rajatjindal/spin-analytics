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
            let xy = req.try_into().expect("cannot convert from Spin HTTP request");
            let recorder = enable_http_analytics(&xy);

            #func

            let result = #func_name(xy);
             let a = match result {
                Ok(resp) => {
                    println!("from inside resp aa");
                    resp.try_into().expect("cannot convert to Spin HTTP response")
                },
                Err(e) => {
                    println!("from inside error");
                    let body = e.to_string();
                        eprintln!("Handler returned an error: {}", body);
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
