use std::iter::Iterator;
use std::time::{SystemTime, UNIX_EPOCH};

pub const QUOTES_STR: &str = include_str!("demo_quotes.txt");

pub type DynQuotesProvider = dyn QuotesProvider + Send + Sync;

#[cfg_attr(test, mockall::automock)]
pub trait QuotesProvider {
    fn random_quote(&self) -> Result<String, String>;
}

pub struct RandomQuotesProvider {
    quotes: Vec<String>
}

impl RandomQuotesProvider {
    pub fn new_boxed() -> Box<DynQuotesProvider> {
        Box::new(Self { quotes: QUOTES_STR.split('\n')
            .map(std::borrow::ToOwned::to_owned)
            .collect() })
    }
}

impl QuotesProvider for RandomQuotesProvider {
    fn random_quote(&self) -> Result<String, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())
            .map(|duration| duration.as_millis() as usize)?;
        self.quotes
            .get(now % self.quotes.len())
            .cloned()
            .ok_or_else(|| "file not found".to_owned())
    }
}
