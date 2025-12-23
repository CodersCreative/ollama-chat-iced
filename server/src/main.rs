#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{Router, middleware};
    use clap::Parser;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use ochat_server::app::*;
    use ochat_server::backend::*;
    use ochat_types::WORD_ART;
    use std::net::SocketAddrV4;
    use std::str::FromStr;

    #[derive(Parser, Debug, Clone)]
    #[command(version, about, long_about = None)]
    struct Arguments {
        #[arg(short, long)]
        url: Option<String>,
    }

    let args = Arguments::parse();
    init_db().await.unwrap();
    let api = Router::new().merge(user::route::auth_routes());

    let api_protected = Router::new()
        .merge(user::route::routes())
        .merge(chats::route::routes())
        .merge(files::route::routes())
        .merge(generation::route::routes())
        .merge(options::route::routes())
        .merge(prompts::route::routes())
        .merge(providers::route::routes())
        .merge(settings::route::routes())
        .route_layer(middleware::from_fn(guard));

    let mut url = args.url.unwrap_or("localhost:1212".to_string());

    if url.is_empty() {
        url = "localhost:1212".to_string();
    }

    url = url.replace("localhost", "127.0.0.1");

    let mut conf = get_configuration(None).unwrap();
    conf.leptos_options.site_addr = std::net::SocketAddr::V4(SocketAddrV4::from_str(&url).unwrap());
    let leptos_options = conf.leptos_options;

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .nest("/api", Router::new().merge(api).merge(api_protected));

    println!("{}", WORD_ART);
    println!("Starting server at '{}'.", url);

    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
