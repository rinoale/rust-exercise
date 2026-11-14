// Typing lecture: macro repetition and optional trailing commas.

use std::collections::HashMap;

macro_rules! count_exprs {
    () => {
        0usize
    };
    ($head:expr $(, $tail:expr)* $(,)?) => {
        1usize + count_exprs!($($tail),*)
    };
}

macro_rules! maplit {
    ($( $key:expr => $value:expr ),* $(,)?) => {{
        let mut map = HashMap::new();
        $(
            map.insert($key, $value);
        )*
        map
    }};
}

fn main() {
    let values: HashMap<&str, i32> = maplit!("one" => 1, "two" => 2);
    println!("{} entries", values.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_expressions() {
        assert_eq!(count_exprs!(), 0);
        assert_eq!(count_exprs!(1), 1);
        assert_eq!(count_exprs!("a", 2 + 3, true,), 3);
    }

    #[test]
    fn builds_hash_maps() {
        let values: HashMap<&str, i32> = maplit!(
            "rust" => 2015,
            "edition" => 2024,
        );

        assert_eq!(values.get("rust"), Some(&2015));
        assert_eq!(values.get("edition"), Some(&2024));
    }
}
