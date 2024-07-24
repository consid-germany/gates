use std::iter::Iterator;
use std::time::{SystemTime, UNIX_EPOCH};

pub const QUOTES_STR: &str = include_str!("demo_quotes.txt");

pub trait QuotesProvider {
    fn new() -> dyn QuotesProvider;
    fn random_quote(&self) -> String;
}

pub struct RandomQuotesProvider {
    quotes: Vec<String>
}

pub struct FakeQuotesProvider {
    quote: String
}

impl QuotesProvider for RandomQuotesProvider {

    fn new() -> Self {
        RandomQuotesProvider { quotes: QUOTES_STR.split("\n").collect() }
    }

    fn random_quote(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time not retrieved")
            .as_millis() as usize;
        (*self.quotes
            .get(now % self.quotes.len())
            .unwrap_or_else(|| panic!("quote could not be obtained")))
            .to_string()
    }

}

impl QuotesProvider for FakeQuotesProvider {

    fn new() -> Self {
        FakeQuotesProvider { quote: "random quote".to_string() }
    }

    fn random_quote(&self) -> String {
        self.quote.to_owned()
    }

}
