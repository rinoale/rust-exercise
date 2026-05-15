// Typing lecture: decorator middleware.

trait Handler {
    fn handle(&self, request: &str) -> String;
}

struct BaseHandler;

impl Handler for BaseHandler {
    fn handle(&self, request: &str) -> String {
        request.to_uppercase()
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
        let Some(rest) = request.strip_prefix("AUTH:") else {
            return String::from("401 Unauthorized");
        };

        self.inner.handle(rest.trim())
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
        eprintln!("request started: {request}");
        let response = self.inner.handle(request);
        eprintln!("request finished");
        response
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
        let _started = std::time::Instant::now();
        self.inner.handle(request)
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
