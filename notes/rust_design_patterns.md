# Rust: Design Patterns

Rust-specific design patterns usually come from:

- ownership
- borrowing
- enums
- traits
- smart pointers
- the type system

Rust design often asks:

```text
Who owns this?
Can it be borrowed?
Can it be mutated?
Can invalid states be represented?
Can the compiler enforce the rule?
```

## RAII / Guard Pattern

RAII means **Resource Acquisition Is Initialization**.

In Rust, a value owns a resource, and the resource is released automatically when the value is dropped.

```rust
use std::sync::Mutex;

fn main() {
    let data = Mutex::new(0);

    {
        let mut guard = data.lock().unwrap();
        *guard += 1;
    } // guard is dropped here, lock is released automatically
}
```

Common examples:

- `String`
- `Vec<T>`
- `Box<T>`
- `File`
- `MutexGuard`
- `RwLockReadGuard`
- `RwLockWriteGuard`

## Enum State Pattern

Instead of inheritance, Rust often models possible states with `enum`.

```rust
enum LoginState {
    LoggedOut,
    LoggedIn { username: String },
}

fn show_status(state: LoginState) {
    match state {
        LoginState::LoggedOut => {
            println!("Please log in");
        }
        LoginState::LoggedIn { username } => {
            println!("Welcome, {}", username);
        }
    }
}
```

This is useful because `match` forces every state to be handled.

## Option / Result Pattern

Rust avoids `null` and exceptions in normal code.

Instead, Rust uses explicit types.

`Option<T>` means:

```text
Some value OR no value
```

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Alice"))
    } else {
        None
    }
}
```

`Result<T, E>` means:

```text
Success with a value OR failure with an error
```

```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("cannot divide by zero"))
    } else {
        Ok(a / b)
    }
}
```

This pattern makes missing values and failures visible in the function type.

## Newtype Pattern

The newtype pattern wraps an existing type in a new type.

This gives stronger meaning and prevents mixing values accidentally.

```rust
struct UserId(u64);
struct ProductId(u64);

fn find_user(id: UserId) {
    println!("Finding user {}", id.0);
}

fn main() {
    let user_id = UserId(1);
    let product_id = ProductId(1);

    find_user(user_id);
    // find_user(product_id); // error
}
```

Useful for:

- IDs
- units
- money
- distances
- domain-specific values

## Builder Pattern

The builder pattern is useful when creating a struct requires many optional settings.

```rust
struct ServerConfig {
    host: String,
    port: u16,
    use_tls: bool,
}

struct ServerConfigBuilder {
    host: String,
    port: u16,
    use_tls: bool,
}

impl ServerConfigBuilder {
    fn new() -> Self {
        Self {
            host: String::from("localhost"),
            port: 8080,
            use_tls: false,
        }
    }

    fn host(mut self, host: String) -> Self {
        self.host = host;
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    fn use_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }

    fn build(self) -> ServerConfig {
        ServerConfig {
            host: self.host,
            port: self.port,
            use_tls: self.use_tls,
        }
    }
}

fn main() {
    let config = ServerConfigBuilder::new()
        .host(String::from("example.com"))
        .port(443)
        .use_tls(true)
        .build();
}
```

This style is common in Rust libraries.

## Typestate Pattern

The typestate pattern uses the type system to prevent invalid states.

```rust
struct Draft;
struct Published;

struct Post<State> {
    title: String,
    state: State,
}

impl Post<Draft> {
    fn publish(self) -> Post<Published> {
        Post {
            title: self.title,
            state: Published,
        }
    }
}

impl Post<Published> {
    fn read(&self) {
        println!("{}", self.title);
    }
}

fn main() {
    let draft = Post {
        title: String::from("Rust patterns"),
        state: Draft,
    };

    // draft.read(); // error: draft posts cannot be read yet

    let published = draft.publish();
    published.read(); // ok
}
```

This makes invalid behavior impossible at compile time.

## Trait-Based Polymorphism

Rust uses traits instead of class inheritance.

```rust
trait Draw {
    fn draw(&self);
}

struct Button;
struct TextBox;

impl Draw for Button {
    fn draw(&self) {
        println!("Drawing button");
    }
}

impl Draw for TextBox {
    fn draw(&self) {
        println!("Drawing text box");
    }
}
```

Static dispatch:

```rust
fn render<T: Draw>(item: T) {
    item.draw();
}
```

Dynamic dispatch:

```rust
fn render(item: &dyn Draw) {
    item.draw();
}
```

Traits are Rust's main abstraction tool.

## Iterator Pattern

Rust often uses iterator chains to transform data.

```rust
let numbers = vec![1, 2, 3, 4, 5];

let doubled_even_numbers: Vec<i32> = numbers
    .iter()
    .filter(|n| *n % 2 == 0)
    .map(|n| n * 2)
    .collect();

println!("{:?}", doubled_even_numbers);
```

This can be more expressive than writing manual loops.

Rust iterators are usually optimized well.

## Interior Mutability Pattern

Normally Rust requires `&mut T` to mutate data.

Sometimes, you need mutation through shared ownership.

For that, Rust provides types such as:

- `Cell<T>`
- `RefCell<T>`
- `Mutex<T>`
- `RwLock<T>`

Example:

```rust
use std::cell::RefCell;

fn main() {
    let value = RefCell::new(10);

    *value.borrow_mut() += 5;

    println!("{}", value.borrow());
}
```

`RefCell<T>` checks borrowing rules at runtime instead of compile time.

If the rules are broken, the program panics.

## Smart Pointer Pattern

Rust uses different smart pointers depending on the ownership need.

| Type | Meaning |
|---|---|
| `Box<T>` | One owner, heap allocation |
| `Rc<T>` | Multiple owners, single-threaded |
| `Arc<T>` | Multiple owners, thread-safe |
| `RefCell<T>` | Runtime borrow checking |
| `Mutex<T>` | Mutable access across threads |

Example:

```rust
use std::rc::Rc;

fn main() {
    let name = Rc::new(String::from("Alice"));

    let a = Rc::clone(&name);
    let b = Rc::clone(&name);

    println!("{}, {}", a, b);
}
```

## Most Rust-Specific Patterns

| Pattern | Rust idea |
|---|---|
| RAII | Cleanup through ownership and `Drop` |
| Enum state pattern | Model states with `enum` |
| `Option` / `Result` | No null, explicit failure |
| Newtype | Stronger type safety |
| Builder | Flexible construction |
| Typestate | Invalid behavior impossible at compile time |
| Traits | Abstraction without inheritance |
| Iterator | Data transformation through iterator chains |
| Interior mutability | Mutate through shared ownership with runtime checks or locks |
| Smart pointers | Explicit ownership strategy |

## Rule Of Thumb

Use enums when you have several possible cases.

Use traits when different types share behavior.

Use newtypes when a primitive type needs stronger meaning.

Use typestate when some operations should only be allowed in certain states.

Use smart pointers when simple ownership is not enough.

Use `Option` and `Result` to make absence and failure explicit.
