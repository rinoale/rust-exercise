// Typing lecture: web framework design.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Method {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Route {
    method: Method,
    path: &'static str,
    action: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Post {
    id: u64,
    title: String,
    body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BlogError {
    NotFound,
    InvalidInput,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusCode {
    Ok,
    Created,
    BadRequest,
    NotFound,
    Conflict,
}

fn blog_routes() -> Vec<Route> {
    vec![
        Route {
            method: Method::Get,
            path: "/posts",
            action: "list_posts",
        },
        Route {
            method: Method::Post,
            path: "/posts",
            action: "create_post",
        },
        Route {
            method: Method::Get,
            path: "/posts/{id}",
            action: "get_post",
        },
        Route {
            method: Method::Put,
            path: "/posts/{id}",
            action: "update_post",
        },
        Route {
            method: Method::Delete,
            path: "/posts/{id}",
            action: "delete_post",
        },
    ]
}

fn status_for_error(error: BlogError) -> StatusCode {
    match error {
        BlogError::NotFound => StatusCode::NotFound,
        BlogError::InvalidInput => StatusCode::BadRequest,
        BlogError::Conflict => StatusCode::Conflict,
    }
}

fn logging_middleware_name() -> &'static str {
    "request_logger"
}

fn main() {
    println!("{:?}", blog_routes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defines_rest_routes() {
        let routes = blog_routes();

        assert_eq!(routes.len(), 5);
        assert!(routes.contains(&Route {
            method: Method::Get,
            path: "/posts",
            action: "list_posts"
        }));
        assert!(routes.contains(&Route {
            method: Method::Post,
            path: "/posts",
            action: "create_post"
        }));
        assert!(routes.contains(&Route {
            method: Method::Get,
            path: "/posts/{id}",
            action: "get_post"
        }));
        assert!(routes.contains(&Route {
            method: Method::Put,
            path: "/posts/{id}",
            action: "update_post"
        }));
        assert!(routes.contains(&Route {
            method: Method::Delete,
            path: "/posts/{id}",
            action: "delete_post"
        }));
    }

    #[test]
    fn maps_errors_to_status_codes() {
        assert_eq!(status_for_error(BlogError::NotFound), StatusCode::NotFound);
        assert_eq!(
            status_for_error(BlogError::InvalidInput),
            StatusCode::BadRequest
        );
        assert_eq!(status_for_error(BlogError::Conflict), StatusCode::Conflict);
    }

    #[test]
    fn names_logging_middleware() {
        assert_eq!(logging_middleware_name(), "request_logger");
    }

    #[test]
    fn models_posts_and_success_statuses() {
        let post = Post {
            id: 1,
            title: String::from("Advanced Rust"),
            body: String::from("typed routes"),
        };

        assert_eq!(post.id, 1);
        assert_eq!(StatusCode::Ok, StatusCode::Ok);
        assert_eq!(StatusCode::Created, StatusCode::Created);
    }
}
