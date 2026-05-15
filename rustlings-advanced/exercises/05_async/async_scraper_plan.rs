// Typing lecture: async planning.
//
// Production code might use tokio, reqwest, a Semaphore, and per-request
// timeouts. This offline exercise models the same flow with a fake async
// client and a tiny test executor.
//
// TODO: Build batches and collect async fetch results.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

#[derive(Debug, Clone, Copy)]
struct ScrapeConfig {
    concurrency_limit: usize,
    timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Page {
    url: String,
    body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScrapeError {
    ZeroConcurrency,
    MissingUrl(String),
    Timeout { url: String, timeout_ms: u64 },
}

struct FakeHttpClient {
    pages: HashMap<String, String>,
    required_timeout_ms: u64,
}

impl FakeHttpClient {
    fn new(pages: impl IntoIterator<Item = (&'static str, &'static str)>) -> Self {
        Self {
            pages: pages
                .into_iter()
                .map(|(url, body)| (url.to_string(), body.to_string()))
                .collect(),
            required_timeout_ms: 50,
        }
    }

    async fn fetch(&self, url: &str, timeout_ms: u64) -> Result<Page, ScrapeError> {
        if timeout_ms < self.required_timeout_ms {
            return Err(ScrapeError::Timeout {
                url: url.to_string(),
                timeout_ms,
            });
        }

        let body = self
            .pages
            .get(url)
            .cloned()
            .ok_or_else(|| ScrapeError::MissingUrl(url.to_string()))?;

        Ok(Page {
            url: url.to_string(),
            body,
        })
    }
}

fn request_batches(
    urls: &[&str],
    concurrency_limit: usize,
) -> Result<Vec<Vec<String>>, ScrapeError> {
    todo!("split URLs into batches of at most concurrency_limit");
}

async fn scrape_all(
    client: &FakeHttpClient,
    urls: &[&str],
    config: ScrapeConfig,
) -> Result<Vec<Page>, ScrapeError> {
    todo!("fetch every URL using the configured batch size and timeout");
}

struct NoopWaker;

impl Wake for NoopWaker {
    fn wake(self: Arc<Self>) {}
}

fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(NoopWaker));
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn main() {
    let client = FakeHttpClient::new([("https://example.test", "ok")]);
    let config = ScrapeConfig {
        concurrency_limit: 2,
        timeout_ms: 100,
    };
    println!(
        "{:?}",
        block_on(scrape_all(&client, &["https://example.test"], config))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_concurrency_batches() {
        let batches = request_batches(&["a", "b", "c", "d", "e"], 2).unwrap();

        assert_eq!(
            batches,
            vec![
                vec![String::from("a"), String::from("b")],
                vec![String::from("c"), String::from("d")],
                vec![String::from("e")]
            ]
        );
        assert_eq!(
            request_batches(&["a"], 0),
            Err(ScrapeError::ZeroConcurrency)
        );
    }

    #[test]
    fn scrapes_pages() {
        let client = FakeHttpClient::new([
            ("https://one.test", "one"),
            ("https://two.test", "two"),
            ("https://three.test", "three"),
        ]);
        let config = ScrapeConfig {
            concurrency_limit: 2,
            timeout_ms: 100,
        };

        let pages = block_on(scrape_all(
            &client,
            &["https://one.test", "https://two.test", "https://three.test"],
            config,
        ))
        .unwrap();

        assert_eq!(pages.len(), 3);
        assert_eq!(pages[1].body, "two");
    }

    #[test]
    fn propagates_errors() {
        let client = FakeHttpClient::new([("https://one.test", "one")]);
        let config = ScrapeConfig {
            concurrency_limit: 1,
            timeout_ms: 10,
        };

        let result = block_on(scrape_all(&client, &["https://one.test"], config));

        assert_eq!(
            result,
            Err(ScrapeError::Timeout {
                url: String::from("https://one.test"),
                timeout_ms: 10
            })
        );
    }
}
