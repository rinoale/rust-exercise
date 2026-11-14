// Typing lecture: generic associated types.

trait LendingStore {
    type View<'a>
    where
        Self: 'a;

    fn get<'a>(&'a self, index: usize) -> Option<Self::View<'a>>;
}

#[derive(Debug, Default)]
struct TextStore {
    lines: Vec<String>,
}

impl TextStore {
    fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

impl LendingStore for TextStore {
    type View<'a>
        = &'a str
    where
        Self: 'a;

    fn get<'a>(&'a self, index: usize) -> Option<Self::View<'a>> {
        self.lines.get(index).map(String::as_str)
    }
}

fn collect_views<'a, S>(store: &'a S, indices: &[usize]) -> Vec<S::View<'a>>
where
    S: LendingStore,
{
    indices
        .iter()
        .filter_map(|&index| store.get(index))
        .collect()
}

fn main() {
    let store = TextStore::new(vec![String::from("gat")]);
    println!("{:?}", store.get(0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_borrowed_views() {
        let store = TextStore::new(vec![
            String::from("first"),
            String::from("second"),
            String::from("third"),
        ]);

        assert_eq!(store.get(1), Some("second"));
        assert_eq!(store.get(99), None);
    }

    #[test]
    fn collects_existing_views() {
        let store = TextStore::new(vec![
            String::from("zero"),
            String::from("one"),
            String::from("two"),
        ]);

        assert_eq!(collect_views(&store, &[2, 0, 99]), vec!["two", "zero"]);
    }
}
