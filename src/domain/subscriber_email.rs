#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: &str) -> Result<Self, String> {
        // TODO: add validation.
        Ok(Self(s.to_owned()))
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
    use claim::assert_err;

    #[test]
    fn empty_string_is_rejected() {
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
