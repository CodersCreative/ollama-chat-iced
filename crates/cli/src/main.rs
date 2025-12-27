use clap::{Args, Parser, Subcommand, ValueEnum};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use ochat_common::{
    data::{Request, RequestType, versions::Versions},
    load_token, save_token,
};
use ochat_types::{
    WORD_ART,
    generation::text::{
        ChatQueryDataBuilder, ChatQueryMessage, ChatQueryMessageBuilder, ChatStreamResult,
    },
    providers::{
        Provider, ProviderData, ProviderDataBuilder, ProviderType,
        ollama::{OllamaModelsInfo, OllamaPullModelStreamResult},
    },
    user::{SigninData, SignupData, Token},
};
use rustyline::{DefaultEditor, error::ReadlineError};
use serde_json::Value;
use std::{
    error::Error,
    io::{self, Write},
    process::Command,
    time::Duration,
};
use tabled::{
    builder::Builder,
    settings::{Alignment, Style},
};

#[derive(Parser, Debug, Clone)]
#[command(about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    action: Option<Action>,
    #[arg(short, long)]
    url: Option<String>,
    #[arg(long)]
    serve: bool,
    #[arg(long)]
    gui: bool,
    #[arg(long)]
    version: bool,
}

#[derive(Subcommand, Debug, Clone)]
enum ProviderAction {
    Run { model: String },
    Pull { model: String },
    Rm { model: String },
    List,
}

#[derive(Subcommand, Debug, Clone)]
enum OllamaAction {
    All { search: Option<String> },
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
    Ollama {
        #[command(subcommand)]
        action: OllamaAction,
    },
    Add {
        name: String,
        url: String,
        r#type: ClapProviderType,
    },
    SignIn {
        name: String,
        password: String,
    },
    SignUp {
        name: String,
        email: String,
        password: String,
    },
    List,
}

#[derive(ValueEnum, Debug, Clone)]
enum ClapProviderType {
    Ollama,
    OpenAI,
    Gemini,
}

impl Into<ProviderType> for ClapProviderType {
    fn into(self) -> ProviderType {
        match self {
            Self::Ollama => ProviderType::Ollama,
            Self::OpenAI => ProviderType::OpenAI,
            Self::Gemini => ProviderType::Gemini,
        }
    }
}

fn spawn_iced() -> Result<std::process::Child, std::io::Error> {
    Command::new("ochat-iced").spawn()
}

fn spawn_server(url: String) -> Result<std::process::Child, std::io::Error> {
    Command::new("ochat-server").arg("--url").arg(url).spawn()
}

async fn run_action(req: &Request, action: Action) -> Result<(), Box<dyn Error>> {
    match action {
        Action::SignIn { name, password } => {
            match req
                .make_request::<Token, SigninData>(
                    "signin/",
                    &SigninData {
                        name: name.clone(),
                        password,
                    },
                    RequestType::Post,
                )
                .await
            {
                Ok(jwt) => {
                    save_token(&jwt);
                    println!("Logged In as '{}'.", name);
                }
                Err(e) => eprintln!("{}", e),
            }
        }
        Action::SignUp {
            name,
            email,
            password,
        } => {
            match req
                .make_request::<Token, SignupData>(
                    "signup/",
                    &SignupData {
                        name: name.clone(),
                        email,
                        password,
                    },
                    RequestType::Post,
                )
                .await
            {
                Ok(jwt) => {
                    save_token(&jwt);
                    println!("Logged In as '{}'.", name);
                }
                Err(e) => eprintln!("{}", e),
            }
        }
        Action::Ollama { action } => match action {
            OllamaAction::All { search } => {
                let models = req
                    .make_request::<Vec<OllamaModelsInfo>, ()>(
                        &if let Some(search) = search {
                            format!("provider/ollama/model/search/{}", search)
                        } else {
                            "provider/ollama/model/all/".to_string()
                        },
                        &(),
                        RequestType::Get,
                    )
                    .await
                    .map(|x| {
                        if x.len() > 100 {
                            x[0..=100].to_vec()
                        } else {
                            x
                        }
                    })
                    .unwrap_or_default();

                let mut table = Builder::new();
                table.push_record(["name", "author"]);

                for model in models {
                    table.push_record([model.name, model.author]);
                }

                let mut table = table.build();
                table.with((Alignment::center(), Style::rounded()));
                print!("{}", table);
            }
        },
        Action::Provider(args) => match args.action {
            ProviderAction::List => {
                let models: Vec<Value> = req
                    .make_request(
                        &format!("provider/{}/model/all/", args.id),
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
            ProviderAction::Pull { model } => pull_model(&req, &args.id, &model).await,
            ProviderAction::Run { model } => {
                let _ = repl(&req, args.id, model).await?;
            }
            ProviderAction::Rm { model } => {
                if let Ok(Some(_)) = req
                    .make_request::<Option<Value>, ()>(
                        &format!("provider/{0}/model/{1}", args.id, model),
                        &(),
                        RequestType::Delete,
                    )
                    .await
                {
                    println!("Successfully deleted {}!", model);
                } else {
                    println!("Failed to delete {}.", model)
                }
            }
        },
        Action::Add { name, url, r#type } => {
            let data = ProviderDataBuilder::default()
                .name(name)
                .url(url)
                .provider_type(r#type.into())
                .build()
                .unwrap();

            let _ = req
                .make_request::<Option<Provider>, ProviderData>(
                    "provider/",
                    &data,
                    RequestType::Post,
                )
                .await;

            let providers: Vec<Provider> = req
                .make_request("provider/all/", &(), RequestType::Get)
                .await?;

            print_providers(providers);
        }
        Action::List => {
            let providers: Vec<Provider> = req
                .make_request("provider/all/", &(), RequestType::Get)
                .await?;

            print_providers(providers);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();
    let mut server_handle = None;

    let url = args.url.clone().unwrap_or("localhost:1212/api".to_string());

    let req = Request {
        url: url.clone(),
        jwt: match load_token() {
            Ok(x) => Some(x.token),
            Err(_) => None,
        },
    };

    if args.serve || (args.action.is_none() && !args.gui && !args.version) {
        server_handle = Some(spawn_server(url.clone())?);
        std::thread::sleep(Duration::from_secs(5));
        println!("Server successfully started at '{}'", url);
    } else {
        println!("{}", WORD_ART);
    }

    if args.version {
        let versions = Versions::get(req.clone()).await;
        println!("latest : {}", versions.latest);
        println!("server : {}", versions.server);
        println!("ochat  : {}", versions.this);

        if let Some(server) = &mut server_handle {
            println!("Closing server.");
            let _ = server.kill()?;
        }

        return Ok(());
    }

    if args.gui || (args.action.is_none() && !args.serve) {
        if args.action.is_some() {
            println!("CLI action will not be performed due to use of the iced gui.");
        }
        println!("Starting iced gui.");

        let mut iced = spawn_iced()?;
        let _ = iced.wait();

        if let Some(server) = &mut server_handle {
            println!("Closing server.");
            let _ = server.kill()?;
        }

        return Ok(());
    }

    if let Some(action) = args.action {
        let res = run_action(&req, action).await;
        if let Some(server) = &mut server_handle {
            println!("Closing server.");
            let _ = server.kill()?;
        }
        return res;
    }

    if let Some(server) = &mut server_handle {
        let _ = server.wait();
    }

    Ok(())
}

async fn repl(req: &Request, provider: String, model: String) -> Result<(), Box<dyn Error>> {
    if let Ok(Some(_)) = req
        .make_request::<Option<Value>, ()>(
            &format!("provider/{0}/model/{1}", provider, model),
            &(),
            RequestType::Get,
        )
        .await
    {
        // Model succesfully retrieved!
    } else {
        println!("Pulling {}.", model);
        pull_model(&req, &provider, &model).await;
    }

    let mut messages: Vec<ChatQueryMessage> = Vec::new();
    let mut editor = DefaultEditor::new()?;

    loop {
        let readline = editor.readline("\n>>> ");
        match readline {
            Ok(line) => {
                editor.add_history_entry(line.clone())?;
                messages.push(
                    ChatQueryMessageBuilder::default()
                        .text(line)
                        .build()
                        .unwrap(),
                );

                let mut response = req
                    .get_client()
                    .get(&format!("{0}/generation/text/stream/", req.url,))
                    .json(
                        &ChatQueryDataBuilder::default()
                            .provider(provider.clone())
                            .model(model.clone())
                            .messages(messages.clone())
                            .build()
                            .unwrap(),
                    )
                    .send()
                    .await
                    .unwrap()
                    .bytes_stream();

                let mut stdout = io::stdout();

                while let Some(response) = response.next().await {
                    match response {
                        Ok(response) => {
                            let _ = match serde_json::from_slice::<ChatStreamResult>(&response) {
                                Ok(x) => match x {
                                    ChatStreamResult::Idle => {}
                                    ChatStreamResult::Generating(x) => {
                                        let _ = stdout.write_all(x.content.as_bytes()).unwrap();
                                        let _ = stdout.flush().unwrap();
                                    }
                                    ChatStreamResult::Generated(x) => {
                                        messages.push(x.into());
                                    }
                                    ChatStreamResult::Finished => {
                                        break;
                                    }
                                    ChatStreamResult::Err(e) => eprintln!("{e}"),
                                },
                                Err(e) => eprintln!("{e}"),
                            };
                        }
                        Err(e) => eprintln!("{e}"),
                    }
                }

                let _ = stdout.write_all(b"\n");
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    Ok(())
}

async fn pull_model(req: &Request, provider: &str, model: &str) {
    let pb = ProgressBar::new(1000000);

    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}%",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    let mut response = req
        .get_client()
        .post(&format!(
            "{0}/provider/{1}/model/{2}",
            req.url, provider, model
        ))
        .send()
        .await
        .unwrap()
        .bytes_stream();

    while let Some(response) = response.next().await {
        match response {
            Ok(response) => {
                let _ = match serde_json::from_slice::<OllamaPullModelStreamResult>(&response) {
                    Ok(x) => match x {
                        OllamaPullModelStreamResult::Idle => {}
                        OllamaPullModelStreamResult::Pulling(x) => {
                            pb.set_position(
                                (x.completed.unwrap_or(0) as f64 / x.total.unwrap_or(1) as f64
                                    * 1000000.0) as u64,
                            );
                        }
                        OllamaPullModelStreamResult::Finished => {
                            pb.finish_with_message(format!("{model} downloaded!"));
                            break;
                        }
                        OllamaPullModelStreamResult::Err(e) => eprintln!("{e}"),
                    },
                    Err(e) => eprintln!("{e}"),
                };
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}

fn print_providers(providers: Vec<Provider>) {
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
