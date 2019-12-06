use crate::stream::{JsonStream, Paginate};
use chrono::{offset::TimeZone, DateTime};
use reqwest::{Client, Error, Url};
use serde_json::Value;

pub const SANDBOX_URL: &str = "https://api-public.sandbox.pro.coinbase.com";

#[derive(Debug)]
pub struct AuthenticatedClient {
    public: MarketData,
}

#[derive(Debug)]
pub struct MarketData {
    client: Client,
    url: Url,
}

impl MarketData {
    pub fn new(url: &str) -> MarketData {
        MarketData {
            client: Client::new(),
            url: Url::parse(url).expect("Invalid Url"),
        }
    }

    pub async fn get_products(&self) -> Result<Value, Error> {
        let url = self.url.join("/products").unwrap();
        self.client.get(url).send().await?.json().await
    }

    pub async fn get_product_order_book(
        &self,
        product_id: &str,
        level: u32,
    ) -> Result<Value, Error> {
        let endpoint = format!("/products/{}/book", product_id);
        let url = self.url.join(&endpoint).unwrap();
        let query = &[("level", &level.to_string()[..])];
        self.client.get(url).query(query).send().await?.json().await
    }

    pub async fn get_product_ticker(&self, product_id: &str) -> Result<Value, Error> {
        let endpoint = format!("/products/{}/ticker", product_id);
        let url = self.url.join(&endpoint).unwrap();
        self.client.get(url).send().await?.json().await
    }

    pub fn get_trades(&self, product_id: &str, limit: u32) -> JsonStream {
        let endpoint = format!("/products/{}/trades", product_id);
        let url = self.url.join(&endpoint).unwrap();
        Paginate::new(self.client.clone(), url.clone(), limit.to_string()).into_json()
    }

    pub async fn get_historic_rates<Tz: TimeZone>(
        &self,
        product_id: &str,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
        granularity: u32,
    ) -> Result<Value, Error>
    where
        Tz::Offset: core::fmt::Display,
    {
        let endpoint = format!("/products/{}/candles", product_id);
        let url = self.url.join(&endpoint).unwrap();
        let query = &[
            ("start", start.to_rfc3339()),
            ("end", end.to_rfc3339()),
            ("granularity", granularity.to_string()),
        ];
        self.client.get(url).query(query).send().await?.json().await
    }

    pub async fn get_24hr_stats(&self, product_id: &str) -> Result<Value, Error> {
        let endpoint = format!("/products/{}/stats", product_id);
        let url = self.url.join(&endpoint).unwrap();
        self.client.get(url).send().await?.json().await
    }

    pub async fn get_currencies(&self) -> Result<Value, Error> {
        let url = self.url.join("/currencies").unwrap();
        self.client.get(url).send().await?.json().await
    }

    pub async fn get_time(&self) -> Result<Value, Error> {
        let url = self.url.join("/time").unwrap();
        self.client.get(url).send().await?.json().await
    }
}
