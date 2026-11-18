fn main() {}

#[allow(unused)]
fn get_greeting(base: &str) -> Result<String, reqwest::Error> {
    reqwest::blocking::get(format!("{base}/hello"))?.text()
}

#[allow(unused)]
fn post_user(base: &str) -> Result<String, reqwest::Error> {
    reqwest::blocking::Client::new()
        .post(format!("{base}/users"))
        .json(&serde_json::json!({
            "name": "foo",
            "email": "bar"
        }))
        .send()?
        .text()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_ok() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/hello")
            .with_status(201)
            .with_header("content-type", "text/plain")
            .with_body("world")
            .create();

        let body = get_greeting(&server.url()).unwrap();
        assert_eq!(body, "world");

        mock.assert();
    }

    #[test]
    fn when_then_ok() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/users")
            .match_header("content-type", "application/json")
            .with_status(201)
            .with_body(r#"{ "id": 1}"#)
            .expect(1)
            .create();

        let body = post_user(&server.url()).unwrap();
        assert_eq!(body, r#"{ "id": 1}"#);

        mock.assert();
    }
}
