// Typing lecture: async-shaped library code.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

#[derive(Debug, Clone, PartialEq, Eq)]
enum FetchError {
    MissingUrl(String),
}

#[derive(Debug, Clone)]
struct FakeClient {
    statuses: HashMap<String, Result<u16, FetchError>>,
}

impl FakeClient {
    fn new(statuses: impl IntoIterator<Item = (&'static str, Result<u16, FetchError>)>) -> Self {
        Self {
            statuses: statuses
                .into_iter()
                .map(|(url, status)| (url.to_string(), status))
                .collect(),
        }
    }

    async fn get_status(&self, url: &str) -> Result<u16, FetchError> {
        self.statuses
            .get(url)
            .cloned()
            .unwrap_or_else(|| Err(FetchError::MissingUrl(url.to_string())))
    }
}

async fn fetch_statuses(client: &FakeClient, urls: &[&str]) -> Result<Vec<u16>, FetchError> {
    let mut statuses = Vec::with_capacity(urls.len());

    for url in urls {
        statuses.push(client.get_status(url).await?);
    }

    Ok(statuses)
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
    let client = FakeClient::new([("https://example.test", Ok(200))]);
    let statuses = block_on(fetch_statuses(&client, &["https://example.test"]));
    println!("{statuses:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_status_codes() {
        let client = FakeClient::new([
            ("https://one.test", Ok(200)),
            ("https://two.test", Ok(204)),
            ("https://three.test", Ok(301)),
        ]);

        let statuses = block_on(fetch_statuses(
            &client,
            &["https://one.test", "https://two.test", "https://three.test"],
        ))
        .unwrap();

        assert_eq!(statuses, vec![200, 204, 301]);
    }

    #[test]
    fn returns_first_fetch_error() {
        let client = FakeClient::new([
            ("https://ok.test", Ok(200)),
            (
                "https://down.test",
                Err(FetchError::MissingUrl(String::from("https://down.test"))),
            ),
        ]);

        let result = block_on(fetch_statuses(
            &client,
            &["https://ok.test", "https://down.test", "https://later.test"],
        ));

        assert_eq!(
            result,
            Err(FetchError::MissingUrl(String::from("https://down.test")))
        );
    }
}
