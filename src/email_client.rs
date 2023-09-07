use reqwest::Client;

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // MockServerは本格的なHTTPサーバーである。
        // MockServer::startはOSに利用可能なランダムなポートを尋ねて、バックグラウンドスレッドでサーバーを起動して、
        // 送信されたリクエストを受信する準備をする。
        let mock_server = MockServer::start().await;
        // MockServer::uriメソッドを使用して、モックサーバーのアドレスを取得できる。
        // そして、EmailClient::newにbase_urlとしてそれを渡すことができる。
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender);

        // 設定しない場合、MockServerは、すべての受信したリクエストに404 Not Foundを返す。
        // Mockをマウントすることによって、異なる振る舞いをするようにモックサーバーに命令できる（ここでは200 OKを返すように指示）。
        // MockServerがリクエストを受け取ったとき、それはマウントされたすべてのモックを順番にチェックして、それらの条件にマッチするか確認する。
        // モックのマッチ条件は、Mock::givenを使用して記述する。
        // Mock::givenにany()を渡しており、それはwiremockドキュメントによると次の通り。
        //      リクエストのメソッド、パス、ヘッダー、ボディに関わらず、すべての受信するリクエストにマッチする。
        //      それについて任意のアサーションをすることなしに、サーバーに向かってリクエストが送信されたかを確認するためにそれを使用できる。
        // expectは、モックサーバーに対して、このテストの期間に、それがこのモックに設定された条件にマッチするリクエストを正確に1つ受け取るべきであることを伝えている。
        // もし、少なくとも1つのリクエスト、expect(1..=3)は、少なくとも1つだが、３以下のリクエストを期待する。
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // アサート（主張）
        // 予期は、MockServerがスコープを外れたときに検証される - 確かに、その場所はテスト関数の末尾である。
        // 終了する前に、MockServerはマウントしたすべてのモックを順番に走査して、それらの予期が検証されたか確認する。
        // もし、検証工程が失敗した場合、それはパニックを引き起こす（そしてテストを失敗する）。
    }
}
