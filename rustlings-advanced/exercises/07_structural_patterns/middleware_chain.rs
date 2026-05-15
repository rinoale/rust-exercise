// Typing lecture: decorator middleware.
//
// Each middleware owns another handler and decides whether to delegate to it.
// This mirrors HTTP middleware stacks without needing a web framework.
//
// TODO: Implement handlers and middleware delegation.

trait Handler {
    fn handle(&self, request: &str) -> String;
}

struct BaseHandler;

impl Handler for BaseHandler {
    fn handle(&self, request: &str) -> String {
        todo!("return the request uppercased");
    }
}

struct AuthMiddleware<H> {
    inner: H,
}

impl<H> AuthMiddleware<H> {
    fn new(inner: H) -> Self {
        Self { inner }
    }
}

impl<H: Handler> Handler for AuthMiddleware<H> {
    fn handle(&self, request: &str) -> String {
        todo!("reject unauthenticated requests or delegate without the AUTH prefix");
    }
}

struct LoggingMiddleware<H> {
    inner: H,
}

impl<H> LoggingMiddleware<H> {
    fn new(inner: H) -> Self {
        Self { inner }
    }
}

impl<H: Handler> Handler for LoggingMiddleware<H> {
    fn handle(&self, request: &str) -> String {
        todo!("log around the request and delegate");
    }
}

struct TimingMiddleware<H> {
    inner: H,
}

impl<H> TimingMiddleware<H> {
    fn new(inner: H) -> Self {
        Self { inner }
    }
}

impl<H: Handler> Handler for TimingMiddleware<H> {
    fn handle(&self, request: &str) -> String {
        todo!("measure or mark timing and delegate");
    }
}

fn app_handler() -> impl Handler {
    TimingMiddleware::new(LoggingMiddleware::new(AuthMiddleware::new(BaseHandler)))
}

fn main() {
    println!("{}", app_handler().handle("AUTH:ping"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_handler_uppercases() {
        assert_eq!(BaseHandler.handle("hello"), "HELLO");
    }

    #[test]
    fn auth_middleware_rejects_missing_prefix() {
        let handler = AuthMiddleware::new(BaseHandler);

        assert_eq!(handler.handle("ping"), "401 Unauthorized");
    }

    #[test]
    fn middleware_chain_delegates_in_order() {
        let handler = app_handler();

        assert_eq!(handler.handle("AUTH:ping"), "PING");
    }
}
