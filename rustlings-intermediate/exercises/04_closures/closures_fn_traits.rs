// Typing lecture: `Fn`, `FnMut`, and `FnOnce`.
//
// Closures implement different traits depending on how they capture values.
// `Fn` reads captured values, `FnMut` mutates captured values, and `FnOnce`
// consumes captured values.
//
// TODO: Implement the helpers using the closure trait bounds.

fn apply_read<F>(input: &str, callback: F) -> String
where
    F: Fn(&str) -> String,
{
    todo!("call a read-only closure");
}

fn apply_three_times<F>(mut value: i32, mut callback: F) -> i32
where
    F: FnMut(i32) -> i32,
{
    todo!("call a mutable closure three times");
}

fn consume_once<F>(input: String, callback: F) -> usize
where
    F: FnOnce(String) -> usize,
{
    todo!("call a consuming closure once");
}

fn make_counter(start: i32) -> impl FnMut() -> i32 {
    let mut current = start;

    move || {
        let _ = &mut current;
        todo!("increment and return the captured count");
    }
}

fn main() {
    let shout = apply_read("rust", |text| text.to_uppercase());
    println!("{shout}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_fn_closure() {
        let suffix = String::from("!");
        let result = apply_read("hello", |text| format!("{text}{suffix}"));

        assert_eq!(result, "hello!");
    }

    #[test]
    fn calls_fn_mut_closure_repeatedly() {
        let mut calls = 0;

        let result = apply_three_times(1, |value| {
            calls += 1;
            value * 2
        });

        assert_eq!(result, 8);
        assert_eq!(calls, 3);
    }

    #[test]
    fn calls_fn_once_closure() {
        let owned = String::from("move me");

        let len = consume_once(owned, |text| text.into_bytes().len());

        assert_eq!(len, 7);
    }

    #[test]
    fn returns_stateful_closure() {
        let mut counter = make_counter(10);

        assert_eq!(counter(), 11);
        assert_eq!(counter(), 12);
        assert_eq!(counter(), 13);
    }
}
