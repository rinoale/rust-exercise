// Typing lecture: advanced lifetimes.

#[derive(Debug, Default)]
struct StringCache {
    text: String,
}

impl StringCache {
    fn new() -> Self {
        Self::default()
    }

    fn store(&mut self, text: String) {
        self.text = text;
    }

    fn view(&self) -> &str {
        &self.text
    }

    fn split_view(&self) -> Vec<&str> {
        self.text.split_whitespace().collect()
    }
}

fn main() {
    let mut cache = StringCache::new();
    cache.store(String::from("advanced lifetimes"));
    println!("{:?}", cache.split_view());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_a_whole_view() {
        let mut cache = StringCache::new();
        cache.store(String::from("borrowed from cache"));

        assert_eq!(cache.view(), "borrowed from cache");
    }

    #[test]
    fn returns_split_views() {
        let mut cache = StringCache::new();
        cache.store(String::from("higher ranked trait bounds"));

        assert_eq!(
            cache.split_view(),
            vec!["higher", "ranked", "trait", "bounds"]
        );
    }

    #[test]
    fn replaces_old_storage() {
        let mut cache = StringCache::new();
        cache.store(String::from("old text"));
        cache.store(String::from("new text"));

        assert_eq!(cache.view(), "new text");
        assert_eq!(cache.split_view(), vec!["new", "text"]);
    }
}
