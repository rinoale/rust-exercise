// Typing lecture: `Fn`, `FnMut`, and `FnOnce`.

fn apply_read<F>(input: &str, callback: F) -> String
where
    F: Fn(&str) -> String,
{
    callback(input)
}

fn apply_three_times<F>(mut value: i32, mut callback: F) -> i32
where
    F: FnMut(i32) -> i32,
{
    for _ in 0..3 {
        value = callback(value);
    }

    value
}

fn consume_once<F>(input: String, callback: F) -> usize
where
    F: FnOnce(String) -> usize,
{
    callback(input)
}

fn make_counter(start: i32) -> impl FnMut() -> i32 {
    let mut current = start;

    move || {
        current += 1;
        current
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
