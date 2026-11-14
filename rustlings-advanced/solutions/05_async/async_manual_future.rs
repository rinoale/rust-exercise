// Typing lecture: manual `Future` and `Pin`.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

#[derive(Debug)]
struct CountDown {
    remaining: u8,
}

impl CountDown {
    fn new(remaining: u8) -> Self {
        Self { remaining }
    }
}

impl Future for CountDown {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();

        if this.remaining == 0 {
            Poll::Ready("done")
        } else {
            this.remaining -= 1;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

struct NoopWaker;

impl Wake for NoopWaker {
    fn wake(self: Arc<Self>) {}
}

fn noop_context() -> Context<'static> {
    let waker = Waker::from(Arc::new(NoopWaker));
    Context::from_waker(Box::leak(Box::new(waker)))
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut context = noop_context();
    let mut future = Box::pin(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn main() {
    println!("{}", block_on(CountDown::new(2)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;

    #[test]
    fn completes_after_countdown() {
        assert_eq!(block_on(CountDown::new(3)), "done");
    }

    #[test]
    fn returns_pending_before_ready() {
        let mut context = noop_context();
        let mut future = Box::pin(CountDown::new(2));

        assert_eq!(Future::poll(future.as_mut(), &mut context), Poll::Pending);
        assert_eq!(Future::poll(future.as_mut(), &mut context), Poll::Pending);
        assert_eq!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready("done")
        );
    }
}
