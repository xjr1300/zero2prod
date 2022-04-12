use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: &str) -> Self {
        let is_empty = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let is_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));
        if is_empty || is_too_long || is_forbidden_chars {
            panic!("{} is not a valid subscriber name.", s)
        } else {
            Self(s.to_owned())
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
