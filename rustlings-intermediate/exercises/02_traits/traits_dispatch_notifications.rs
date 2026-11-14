// Typing lecture: trait default methods and dispatch.
//
// Rust uses traits for shared behavior. Generic functions use static dispatch,
// while `dyn Trait` values use dynamic dispatch through trait objects.
//
// TODO: Implement the notifiers and the two broadcast helpers.

trait Notifier {
    fn channel(&self) -> &'static str;

    fn send(&self, message: &str) -> String;

    fn summary(&self, message: &str) -> String {
        format!("[{}] {}", self.channel(), message)
    }
}

struct Email {
    address: String,
}

struct Sms {
    number: String,
}

impl Email {
    fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
        }
    }
}

impl Sms {
    fn new(number: &str) -> Self {
        Self {
            number: number.to_string(),
        }
    }
}

impl Notifier for Email {
    fn channel(&self) -> &'static str {
        todo!("return the email channel name");
    }

    fn send(&self, message: &str) -> String {
        todo!("format an email delivery string");
    }
}

impl Notifier for Sms {
    fn channel(&self) -> &'static str {
        todo!("return the sms channel name");
    }

    fn send(&self, message: &str) -> String {
        todo!("format an sms delivery string");
    }
}

fn broadcast_static<T>(notifier: &T, message: &str) -> String
where
    T: Notifier,
{
    todo!("send a message through a statically dispatched notifier");
}

fn broadcast_dynamic(notifiers: &[Box<dyn Notifier>], message: &str) -> Vec<String> {
    todo!("send the message through every boxed trait object");
}

fn channel_names(notifiers: &[&dyn Notifier]) -> Vec<&'static str> {
    todo!("collect channel names from borrowed trait objects");
}

fn main() {
    let email = Email::new("team@example.test");
    println!("{}", broadcast_static(&email, "hello"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_default_trait_method() {
        let email = Email::new("dev@example.test");

        assert_eq!(email.summary("build passed"), "[email] build passed");
    }

    #[test]
    fn statically_dispatches_generic_notifiers() {
        let sms = Sms::new("+10000000000");

        assert_eq!(broadcast_static(&sms, "deploy"), "sms:+10000000000:deploy");
    }

    #[test]
    fn dynamically_dispatches_trait_objects() {
        let notifiers: Vec<Box<dyn Notifier>> = vec![
            Box::new(Email::new("ops@example.test")),
            Box::new(Sms::new("+12223334444")),
        ];

        assert_eq!(
            broadcast_dynamic(&notifiers, "alert"),
            vec![
                String::from("email:ops@example.test:alert"),
                String::from("sms:+12223334444:alert"),
            ]
        );
    }

    #[test]
    fn borrows_dyn_trait_values() {
        let email = Email::new("ops@example.test");
        let sms = Sms::new("+12223334444");
        let notifiers: Vec<&dyn Notifier> = vec![&email, &sms];

        assert_eq!(channel_names(&notifiers), vec!["email", "sms"]);
    }
}
