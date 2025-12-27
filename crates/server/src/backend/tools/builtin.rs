use std::{collections::HashMap, sync::Arc};

use rig::completion::ToolDefinition;
use rig::tool::*;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;
use spider::website::Website;
use tokio::sync::Mutex;
use websearch::{SearchResult, providers::DuckDuckGoProvider, web_search};

use crate::backend::errors::ServerError;

#[derive(Deserialize, Serialize)]
pub struct WebSearch;

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct WebSearchArgs {
    pub query: String,
    pub max_results: Option<u32>,
}

#[derive(Deserialize, Serialize)]
pub struct WebSearchResult(Vec<SearchResult>);

impl ToolEmbedding for WebSearch {
    type InitError = ServerError;
    type Context = ();
    type State = ();

    fn init(_state: Self::State, _context: Self::Context) -> Result<Self, Self::InitError> {
        Ok(Self)
    }

    fn embedding_docs(&self) -> Vec<String> {
        vec!["Search the internet using a query".into()]
    }

    fn context(&self) -> Self::Context {}
}

impl Tool for WebSearch {
    const NAME: &'static str = "web_search";
    type Error = ServerError;
    type Args = WebSearchArgs;
    type Output = WebSearchResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": Self::NAME,
            "description": "Search the internet using a query",
            "parameters": schema_for!(Self::Args)
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("Calling {}", Self::NAME);
        let result = web_search(websearch::SearchOptions {
            query: args.query,
            max_results: Some(args.max_results.unwrap_or(10)),
            safe_search: Some(websearch::types::SafeSearch::Off),
            provider: Box::new(DuckDuckGoProvider::new()),
            ..Default::default()
        })
        .await?;
        Ok(WebSearchResult(result))
    }
}

#[derive(Deserialize, Serialize)]
pub struct WebScraper;

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct WebScraperArgs {
    pub url: String,
    pub include_subdomains: Option<bool>,
    pub max_pages: Option<u32>,
}

#[derive(Deserialize, Serialize)]
pub struct WebScraperResult(HashMap<String, String>);

impl ToolEmbedding for WebScraper {
    type InitError = ServerError;
    type Context = ();
    type State = ();

    fn init(_state: Self::State, _context: Self::Context) -> Result<Self, Self::InitError> {
        Ok(Self)
    }

    fn embedding_docs(&self) -> Vec<String> {
        vec!["Scrape a website or page for information".into()]
    }

    fn context(&self) -> Self::Context {}
}

impl Tool for WebScraper {
    const NAME: &'static str = "web_scraper";
    type Error = ServerError;
    type Args = WebScraperArgs;
    type Output = WebScraperResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": Self::NAME,
            "description": "Scrape a website or page for information",
            "parameters": schema_for!(Self::Args)
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("Calling {}", Self::NAME);
        let mut website = Website::new(&args.url);
        let mut site = website.with_respect_robots_txt(true);
        if let Some(x) = args.include_subdomains {
            site = site.with_subdomains(x);
        }

        if let Some(x) = args.max_pages {
            site = site.with_limit(x);
        }

        let pages = Arc::new(Mutex::new(HashMap::new()));

        {
            let mut rx2 = site.subscribe(16).unwrap();
            let pages_rx = pages.clone();

            tokio::spawn(async move {
                while let Ok(res) = rx2.recv().await {
                    pages_rx
                        .lock()
                        .await
                        .insert(res.get_url().to_string(), res.get_html());
                }
            });
        }

        site.crawl().await;
        site.unsubscribe();

        Ok(WebScraperResult(pages.lock().await.clone()))
    }
}
