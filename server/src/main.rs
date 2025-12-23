#[cfg(feature = "normal")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use ochat_server::backend::start_server;

    #[cfg(feature = "ssr")]
    {
        use leptos::prelude::*;
        use leptos_axum::{LeptosRoutes, generate_route_list};
        use ochat_server::app::*;
        use std::net::SocketAddrV4;
        use std::str::FromStr;

        let _ = start_server(|url| {
            let mut conf = get_configuration(None).unwrap();
            conf.leptos_options.site_addr =
                std::net::SocketAddr::V4(SocketAddrV4::from_str(&url).unwrap());
            let leptos_options = conf.leptos_options;

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(App);

            Router::new()
                .leptos_routes(&leptos_options, routes, {
                    let leptos_options = leptos_options.clone();
                    move || shell(leptos_options.clone())
                })
                .fallback(leptos_axum::file_and_error_handler(shell))
                .with_state(leptos_options)
        })
        .await;
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = start_server(|_| Router::new()).await;
    }
}

#[cfg(not(feature = "normal"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
