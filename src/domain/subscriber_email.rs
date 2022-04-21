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

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

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

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        // dbg!(&valid_email.0);
        SubscriberEmail::parse(&valid_email.0).is_ok()
    }
}
