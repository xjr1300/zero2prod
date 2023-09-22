#[derive(Debug)]
pub struct IdempotencyKey(String);

impl TryFrom<String> for IdempotencyKey {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Err(anyhow::anyhow!("冪等キーが空になることはありません。"));
        }
        let max_length = 50;
        if s.len() >= max_length {
            return Err(anyhow::anyhow!(
                "冪等キーは{}文字を超えることはできません。",
                max_length
            ));
        }

        Ok(Self(s))
    }
}

impl From<IdempotencyKey> for String {
    fn from(value: IdempotencyKey) -> Self {
        value.0
    }
}

impl AsRef<str> for IdempotencyKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
