// Typing lecture: advanced lifetimes.
//
// The cache owns a `String`. Views returned from it borrow from `self`, so they
// cannot outlive the cache. This is the same relationship used by parsers and
// other libraries that expose borrowed views into owned storage.
//
// TODO: Store text and return borrowed views into it.

#[derive(Debug, Default)]
struct StringCache {
    text: String,
}

impl StringCache {
    fn new() -> Self {
        Self::default()
    }

    fn store(&mut self, text: String) {
        todo!("replace the cached string");
    }

    fn view(&self) -> &str {
        todo!("return the whole cached string as a borrowed str");
    }

    fn split_view(&self) -> Vec<&str> {
        todo!("return borrowed word slices");
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
