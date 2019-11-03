use reqwest::{Client, Response, Result};
use serde_json;

pub fn post(url: &reqwest::Url, js: &serde_json::Value) -> Result<Response> {
    let client = Client::new();
    client.post(url.clone()).json(js).send()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use mockito::{mock, Matcher};
    use serde_json::json;

    #[test]
    fn test_post() {
        let host = &mockito::server_url();

        let url = reqwest::Url::parse(host).unwrap();
        let js = json!({"hello":"world"});

        let mock = mock("POST", "/")
            .match_body(Matcher::Json(js.clone()))
            .create();
        let _t = post(&url, &js);
        mock.assert();
    }
}
