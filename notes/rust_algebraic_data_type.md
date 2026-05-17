# Rust: Algebraic Data Type

An **algebraic data type**, or **ADT**, is a way to build types by combining other types.

In Rust, ADTs mainly appear as:

| ADT kind | Rust feature | Meaning |
|---|---|---|
| Product type | `struct`, tuple | This **and** that |
| Sum type | `enum` | This **or** that |

## Product Type: Struct

A product type means the value contains **multiple pieces together**.

```rust
struct User {
    name: String,
    age: u8,
}
```

A `User` has:

```text
name AND age
```

Example:

```rust
let user = User {
    name: String::from("Alice"),
    age: 30,
};
```

So a `struct` is like a bundle of fields.

Another example:

```rust
struct Point {
    x: i32,
    y: i32,
}
```

A `Point` has:

```text
x AND y
```

## Tuple As Product Type

A tuple is also a product type.

```rust
let point: (i32, i32) = (10, 20);
```

This means the value has:

```text
i32 AND i32
```

Structs are usually clearer than tuples when each field has a meaning.

## Sum Type: Enum

A sum type means the value can be **one of several possibilities**.

```rust
enum PaymentMethod {
    Cash,
    CreditCard(String),
    BankTransfer { account_number: String },
}
```

A `PaymentMethod` is:

```text
Cash OR CreditCard OR BankTransfer
```

Each variant can carry different data.

Example:

```rust
let payment1 = PaymentMethod::Cash;

let payment2 = PaymentMethod::CreditCard(String::from("1234-5678"));

let payment3 = PaymentMethod::BankTransfer {
    account_number: String::from("ABC-999"),
};
```

## Pattern Matching

ADTs are usually used with `match`.

```rust
enum PaymentMethod {
    Cash,
    CreditCard(String),
    BankTransfer { account_number: String },
}

fn pay(method: PaymentMethod) {
    match method {
        PaymentMethod::Cash => {
            println!("Paid with cash");
        }
        PaymentMethod::CreditCard(card_number) => {
            println!("Paid with credit card: {}", card_number);
        }
        PaymentMethod::BankTransfer { account_number } => {
            println!("Paid by bank transfer: {}", account_number);
        }
    }
}

fn main() {
    let method = PaymentMethod::CreditCard(String::from("1234-5678"));
    pay(method);
}
```

Rust forces you to handle every possible variant in a `match`.

That is one reason enums are powerful and safe.

## Common Rust ADT: Option

`Option<T>` is an enum.

It means:

```text
There is a value OR there is no value
```

It is roughly defined like this:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

Example:

```rust
let name: Option<String> = Some(String::from("Alice"));
let missing_name: Option<String> = None;
```

Using it:

```rust
fn print_name(name: Option<String>) {
    match name {
        Some(value) => println!("Name: {}", value),
        None => println!("No name"),
    }
}
```

`Option<T>` is useful when a value may or may not exist.

## Common Rust ADT: Result

`Result<T, E>` is also an enum.

It means:

```text
Success with a value OR failure with an error
```

It is roughly defined like this:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Example:

```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("Cannot divide by zero"))
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10, 2) {
        Ok(value) => println!("Result: {}", value),
        Err(error) => println!("Error: {}", error),
    }
}
```

`Result<T, E>` is useful when an operation can succeed or fail.

## Simple Mental Model

```rust
struct Person {
    name: String,
    age: u8,
}
```

Means:

```text
Person = name AND age
```

```rust
enum LoginState {
    LoggedOut,
    LoggedIn(String),
}
```

Means:

```text
LoginState = LoggedOut OR LoggedIn with username
```

## Rule Of Thumb

Use a `struct` when one value should contain several fields together.

Use an `enum` when one value should be one of several possible cases.

Use `Option<T>` when a value may be missing.

Use `Result<T, E>` when an operation may fail.
