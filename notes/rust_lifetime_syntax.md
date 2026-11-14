# Rust: Lifetime Syntax `<'a>` And `<'_>`

`<'a>` and `<'_>` are **lifetime parameters** in Rust.

They are used when a type contains references and Rust needs to know how long those references are valid.

## Named Lifetime: `<'a>`

`'a` is a lifetime name.

Example:

```rust
struct User<'a> {
    name: &'a str,
}
```

This means:

```text
User contains a string reference.
That reference must be valid for lifetime 'a.
```

Usage:

```rust
fn main() {
    let name = String::from("Alice");

    let user = User {
        name: &name,
    };

    println!("{}", user.name);
}
```

`User<'a>` does **not** own the string.

It only borrows it.

So this would be invalid:

```rust
let user;

{
    let name = String::from("Alice");
    user = User { name: &name };
} // name is dropped here

// println!("{}", user.name); // error
```

This is invalid because `user.name` would point to data that no longer exists.

## Named Lifetimes In Functions

Named lifetimes are useful when references are related.

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

This means:

```text
The returned reference is valid only as long as both input references are valid.
```

`'a` connects the lifetimes of:

- `x`
- `y`
- the returned reference

## Anonymous Inferred Lifetime: `<'_>`

`'_` means:

```text
There is a lifetime here, but I do not want to name it.
Compiler, please infer it.
```

Example:

```rust
fn print_user(user: &User<'_>) {
    println!("{}", user.name);
}
```

`User<'_>` means:

```text
User has some lifetime, but this function does not need to name it.
```

Another common example:

```rust
use std::fmt;

impl fmt::Display for User<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
```

Here:

```rust
fmt::Formatter<'_>
```

means:

```text
Formatter has a lifetime parameter,
but this code does not need to name it.
```

## Difference

| Syntax | Meaning |
|---|---|
| `<'a>` | Named lifetime |
| `<'_>` | Anonymous inferred lifetime |
| `&'a T` | Reference valid for named lifetime `'a` |
| `&'_ T` | Reference with inferred anonymous lifetime |

## When To Use `<'a>`

Use `<'a>` when you need to connect lifetimes or refer to the same lifetime more than once.

Example:

```rust
struct Book<'a> {
    title: &'a str,
}

fn get_title<'a>(book: &'a Book<'a>) -> &'a str {
    book.title
}
```

Here, `'a` connects the lifetime of:

- the `Book`
- the borrowed title
- the returned reference

## When To Use `<'_>`

Use `<'_>` when the lifetime exists but you do not need to talk about it.

Example:

```rust
fn print_user(user: &User<'_>) {
    println!("{}", user.name);
}
```

This keeps the code shorter when naming the lifetime would not add useful information.

## Why Lifetimes Are Useful

Lifetimes do **not** make data live longer.

Lifetimes let you describe relationships between references so Rust can check them safely.

Rust already tracks lifetimes internally. You write lifetimes only when Rust needs help understanding how references are connected.

Lifetimes are a **compile-time constraint**.

They are not runtime objects.

They do not add runtime checks.

They help the compiler reject invalid reference usage before the program runs.

The main benefit:

```text
Lifetimes let Rust allow fast borrowing while preventing dangling references.
```

They help Rust provide:

- no garbage collector
- no manual `free`
- no dangling references
- less unnecessary cloning

In other languages, a bad reference might become:

- runtime crash
- use-after-free bug
- undefined behavior
- security bug

In Rust, lifetimes help make that kind of mistake a compile error instead.

Example:

```rust
struct User<'a> {
    name: &'a str,
}

fn main() {
    let user;

    {
        let name = String::from("gbsong");
        user = User { name: &name };
    } // name is dropped here

    println!("{}", user.name); // compile error
}
```

Rust rejects this because `user.name` would point to data that has already been dropped.

So the mental model is:

```text
Lifetimes are compile-time rules for reference safety.
They make programs more robust by preventing invalid references before runtime.
```

## What Does Not Compile Without Lifetimes

This function does not compile:

```rust
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

Rust sees that the return value could come from `x` or `y`.

But it does not know how long the returned reference should be valid.

With an explicit lifetime:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

This means:

```text
The returned reference is valid only as long as both input references are valid.
```

The lifetime `'a` connects:

- `x`
- `y`
- the returned reference

## What Lifetimes Prevent

Lifetimes prevent references from pointing to data that has already been dropped.

Example:

```rust
let result;

let a = String::from("hello");

{
    let b = String::from("world");
    result = longest(&a, &b);
} // b is dropped here

// println!("{}", result); // error
```

Rust rejects this because `result` might refer to `b`, but `b` is already gone.

The lifetime rule is:

```text
The result cannot live longer than the shortest input reference.
```

## Structs That Borrow Data

Without a lifetime, this does not compile:

```rust
struct User {
    name: &str,
}
```

Rust needs to know how long `name` is valid.

With a lifetime:

```rust
struct User<'a> {
    name: &'a str,
}
```

This means:

```text
User cannot live longer than the string it borrows.
```

Valid example:

```rust
let name = String::from("Alice");

let user = User {
    name: &name,
};

println!("{}", user.name);
```

Invalid example:

```rust
let user;

{
    let name = String::from("Alice");
    user = User { name: &name };
} // name is dropped here

// println!("{}", user.name); // error
```

Rust rejects this because `user.name` would point to dropped data.

## What Lifetimes Make Possible

Lifetimes make it possible to store references in structs safely.

Example:

```rust
struct Parser<'a> {
    input: &'a str,
}
```

The parser can read from existing text without copying it.

Another example:

```rust
struct Document<'a> {
    title: &'a str,
    body: &'a str,
}
```

The document can borrow existing strings instead of owning new `String` values.

Without borrowing, you might need to clone or own the data:

```rust
struct User {
    name: String,
}
```

With lifetimes, the struct can borrow:

```rust
struct User<'a> {
    name: &'a str,
}
```

This can avoid extra allocation and copying.

## Lifetime Relationships

Lifetimes let you express:

```text
This returned reference comes from this input.
```

Example:

```rust
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
```

Rust can infer this lifetime because there is only one input reference.

Lifetimes also let you express:

```text
This output cannot outlive this input.
```

Example:

```rust
fn get_name<'a>(user: &'a User<'a>) -> &'a str {
    user.name
}
```

## Simple Mental Model

```text
<'a> = I need to name this lifetime because I will refer to it.
<'_> = There is a lifetime here, but Rust can infer it.
```

Most beginner Rust code does not need explicit lifetimes often because Rust can usually infer them.

The deeper mental model:

```text
Without lifetimes, Rust cannot safely answer:
- What does this reference point to?
- How long is that data alive?
- Can this reference still be used here?

With lifetimes, you tell Rust:
- these references are connected
- this output reference depends on this input reference
- this struct cannot outlive the data it borrows
```
