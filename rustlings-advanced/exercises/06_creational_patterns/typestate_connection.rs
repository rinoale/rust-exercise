// Typing lecture: typestate.
//
// Typestate uses types to encode which operations are valid. A disconnected
// connection can connect, but it cannot query. An authenticated connection can
// query, but it does not expose authentication again.
//
// TODO: Implement the valid state transitions.

use std::marker::PhantomData;

struct Disconnected;
struct Connecting;
struct Connected;
struct Authenticated;

#[derive(Debug, PartialEq, Eq)]
enum AuthError {
    BadPassword,
}

struct Connection<State> {
    endpoint: String,
    user: Option<String>,
    _state: PhantomData<State>,
}

impl Connection<Disconnected> {
    fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            user: None,
            _state: PhantomData,
        }
    }

    fn connect(self) -> Connection<Connecting> {
        todo!("move from Disconnected to Connecting");
    }
}

impl Connection<Connecting> {
    fn on_connected(self) -> Connection<Connected> {
        todo!("move from Connecting to Connected");
    }

    fn on_error(self) -> Connection<Disconnected> {
        todo!("move back to Disconnected");
    }
}

impl Connection<Connected> {
    fn authenticate(
        self,
        user: &str,
        password: &str,
    ) -> Result<Connection<Authenticated>, AuthError> {
        todo!("authenticate only when connected");
    }
}

impl Connection<Authenticated> {
    fn query(&self, sql: &str) -> String {
        todo!("run a query as the authenticated user");
    }

    fn disconnect(self) -> Connection<Disconnected> {
        todo!("disconnect and return to Disconnected");
    }
}

fn main() {
    let connection = Connection::new("db://localhost").connect().on_connected();
    println!("connected to {}", connection.endpoint);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_valid_happy_path() {
        let connection = Connection::new("db://localhost")
            .connect()
            .on_connected()
            .authenticate("gbsong", "secret")
            .unwrap();

        assert_eq!(
            connection.query("select 1"),
            "gbsong@db://localhost: select 1"
        );

        let disconnected = connection.disconnect();
        assert_eq!(disconnected.endpoint, "db://localhost");
    }

    #[test]
    fn can_return_to_disconnected_on_connect_error() {
        let disconnected = Connection::new("db://localhost").connect().on_error();

        assert_eq!(disconnected.endpoint, "db://localhost");
    }

    #[test]
    fn rejects_bad_password() {
        let result = Connection::new("db://localhost")
            .connect()
            .on_connected()
            .authenticate("gbsong", "wrong");

        assert_eq!(result.err(), Some(AuthError::BadPassword));
    }
}
