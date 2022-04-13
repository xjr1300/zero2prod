use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: &str) -> Result<Self, String> {
        if validate_email(s) {
            Ok(Self(s.to_owned()))
        } else {
            Err(format!("{} is not a valid subscriber email.", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok};

    #[test]
    fn email_is_accepted() {
        assert_ok!(SubscriberEmail::parse("anonymous-domain@domain.com"));
    }

    #[test]
    fn email_empty_is_rejected() {
        assert_err!(SubscriberEmail::parse(""));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        assert_err!(SubscriberEmail::parse("anonymous-domain.com"));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        assert_err!(SubscriberEmail::parse("@domain.com"));
    }
}
