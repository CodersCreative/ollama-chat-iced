pub mod data;

use clap::{Args, Parser, Subcommand};
use ochat_types::providers::Provider;
use serde_json::Value;
use std::error::Error;
use tabled::{
    builder::Builder,
    settings::{Alignment, Style},
};

use crate::data::RequestType;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    action: Action,
    #[arg(short, long)]
    url: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
enum ProviderArgType {
    Name(ProviderName),
    Id(ProviderId),
}

#[derive(Args, Debug, Clone)]
pub struct ProviderName {
    name: String,
}

#[derive(Args, Debug, Clone)]
pub struct ProviderId {
    id: String,
}

#[derive(Subcommand, Debug, Clone)]
enum ProviderAction {
    Run { model: String },
    Pull { model: String },
    Rm { model: String },
    List,
}

#[derive(Args, Debug, Clone)]
struct ProviderArgs {
    id: String,
    #[command(subcommand)]
    action: ProviderAction,
}

#[derive(Subcommand, Debug, Clone)]
enum Action {
    Provider(ProviderArgs),
    List,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();

    let req = if let Some(url) = args.url {
        data::Request(url)
    } else {
        data::Request(String::from("http://localhost:1212"))
    };

    match args.action {
        Action::Provider(arg) => match arg.action {
            ProviderAction::List => {
                let models: Vec<Value> = req
                    .make_request(
                        &format!("provider/{}/model/all/", arg.id),
                        &(),
                        RequestType::Get,
                    )
                    .await
                    .unwrap();

                let mut table = Builder::new();
                table.push_record(["model"]);

                for model in models {
                    table.push_record([model["id"].as_str().unwrap().to_string()]);
                }

                let mut table = table.build();
                table.with((Alignment::center(), Style::rounded()));
                print!("{}", table);
            }
            _ => unimplemented!(),
        },
        Action::List => {
            let providers: Vec<Provider> = req
                .make_request("provider/all/", &(), RequestType::Get)
                .await?;
            let mut table = Builder::new();
            table.push_record(["id", "name", "url", "type"]);

            for provider in providers {
                table.push_record([
                    provider.id.key().to_string(),
                    provider.name,
                    provider.url,
                    provider.provider_type.to_string(),
                ]);
            }

            let mut table = table.build();
            table.with((Alignment::center(), Style::rounded()));
            print!("{}", table);
        }
    }

    Ok(())
}
