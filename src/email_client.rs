use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        // 7.2.4.1で、reqwest::Urlを使用してURLを構築すると、エラーが発生した場合、
        // urlクレートのParseErrorをエラーとして返すため、send_emailメソッドの戻り値
        // であるResultのエラー型が一致せずにコンパイルエラーが発生する。
        // この場合、anyhowクレートを導入するなど、複数のエラー型に対応する必要がある。
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };
        self.http_client
            .post(url.as_str())
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                dbg!(&body);
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_uri: String) -> EmailClient {
        EmailClient::new(
            base_uri,
            email(),
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // MockServerは本格的なHTTPサーバーである。
        // MockServer::startはOSに利用可能なランダムなポートを尋ねて、バックグラウンドスレッドでサーバーを起動して、
        // 送信されたリクエストを受信する準備をする。
        let mock_server = MockServer::start().await;
        // MockServer::uriメソッドを使用して、モックサーバーのアドレスを取得できる。
        // そして、EmailClient::newにbase_urlとしてそれを渡すことができる。
        let email_client = email_client(mock_server.uri());

        // 設定しない場合、MockServerは、すべての受信したリクエストに404 Not Foundを返す。
        // Mockをマウントすることによって、異なる振る舞いをするようにモックサーバーに命令できる（ここでは200 OKを返すように指示）。
        // MockServerがリクエストを受け取ったとき、それはマウントされたすべてのモックを順番にチェックして、それらの条件にマッチするか確認する。
        // モックのマッチ条件は、Mock::givenを使用して記述する。
        // Mock::givenにany()を渡しており、それはwiremockドキュメントによると次の通り。
        //      リクエストのメソッド、パス、ヘッダー、ボディに関わらず、すべての受信するリクエストにマッチする。
        //      それについて任意のアサーションをすることなしに、サーバーに向かってリクエストが送信されたかを確認するためにそれを使用できる。
        // expectは、モックサーバーに対して、このテストの期間に、それがこのモックに設定された条件にマッチするリクエストを正確に1つ受け取るべきであることを伝えている。
        // もし、少なくとも1つのリクエスト、expect(1..=3)は、少なくとも1つだが、３以下のリクエストを期待する。
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // アサート（主張）
        // 予期は、MockServerがスコープを外れたときに検証される - 確かに、その場所はテスト関数の末尾である。
        // 終了する前に、MockServerはマウントしたすべてのモックを順番に走査して、それらの予期が検証されたか確認する。
        // もし、検証工程が失敗した場合、それはパニックを引き起こす（そしてテストを失敗する）。
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }
}
