// Typing lecture: typestate.

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
        Connection {
            endpoint: self.endpoint,
            user: self.user,
            _state: PhantomData,
        }
    }
}

impl Connection<Connecting> {
    fn on_connected(self) -> Connection<Connected> {
        Connection {
            endpoint: self.endpoint,
            user: self.user,
            _state: PhantomData,
        }
    }

    fn on_error(self) -> Connection<Disconnected> {
        Connection {
            endpoint: self.endpoint,
            user: None,
            _state: PhantomData,
        }
    }
}

impl Connection<Connected> {
    fn authenticate(
        self,
        user: &str,
        password: &str,
    ) -> Result<Connection<Authenticated>, AuthError> {
        if password != "secret" {
            return Err(AuthError::BadPassword);
        }

        Ok(Connection {
            endpoint: self.endpoint,
            user: Some(user.to_string()),
            _state: PhantomData,
        })
    }
}

impl Connection<Authenticated> {
    fn query(&self, sql: &str) -> String {
        let user = self.user.as_deref().unwrap_or("unknown");
        format!("{user}@{}: {sql}", self.endpoint)
    }

    fn disconnect(self) -> Connection<Disconnected> {
        Connection {
            endpoint: self.endpoint,
            user: None,
            _state: PhantomData,
        }
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
