use std::collections::HashMap;
use std::time::Duration;

use chrono::Utc;
use hmac::{Hmac, Mac};
use percent_encoding::{percent_encode, AsciiSet, CONTROLS};
use reqwest::{Client, Method};
use sha2::{Digest, Sha256};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[tokio::main]
async fn main() {
    let response = send_sms_verify_code("", "", "", "", "", "{\"code\":\"3306\",\"min\":\"5\"}")
        .await
        .unwrap();

    println!("Response: {:?}", response);
}

async fn send_sms_verify_code(
    access_key_id: &str,
    access_key_secret: &str,
    phone_number: &str,
    sign_name: &str,
    template_code: &str,
    template_param: &str,
) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
    let host = "dypnsapi.aliyuncs.com";
    let action = "SendSmsVerifyCode";
    let version = "2017-05-25";
    let algorithm = "ACS3-HMAC-SHA256";
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let nonce = Uuid::new_v4().to_string();

    let query_params = vec![
        ("PhoneNumber", phone_number),
        ("SignName", sign_name),
        ("TemplateCode", template_code),
        ("TemplateParam", template_param),
    ];

    let method = "POST";
    let canonical_uri = "/";
    let canonical_query_string = generate_canonical_query_string(&query_params);
    let hashed_request_payload = sha256_hex(b"");
    let canonical_headers = format!(
        "host:{host}\nx-acs-action:{action}\nx-acs-content-sha256:{hashed_request_payload}\nx-acs-date:{timestamp}\nx-acs-signature-nonce:{nonce}\nx-acs-version:{version}\n"
    );
    let signed_headers =
        "host;x-acs-action;x-acs-content-sha256;x-acs-date;x-acs-signature-nonce;x-acs-version";
    let canonical_request = format!(
        "{method}\n{canonical_uri}\n{canonical_query_string}\n{canonical_headers}\n{signed_headers}\n{hashed_request_payload}"
    );
    let string_to_sign = format!("{algorithm}\n{}", sha256_hex(canonical_request.as_bytes()));
    let signature = hmac_sha256_hex(access_key_secret.as_bytes(), string_to_sign.as_bytes());
    let authorization = format!(
        "{algorithm} Credential={access_key_id},SignedHeaders={signed_headers},Signature={signature}"
    );

    let response = Client::new()
        .request(Method::POST, format!("https://{host}/"))
        .query(&query_params)
        .timeout(Duration::from_millis(3000))
        .header("Authorization", authorization)
        .header("host", host)
        .header("x-acs-action", action)
        .header("x-acs-content-sha256", hashed_request_payload)
        .header("x-acs-date", timestamp)
        .header("x-acs-version", version)
        .header("x-acs-signature-nonce", nonce)
        .send()
        .await?
        .json::<HashMap<String, serde_json::Value>>()
        .await?;

    Ok(response)
}

fn sha256_hex(message: &[u8]) -> String {
    hex::encode(Sha256::digest(message))
}

fn hmac_sha256_hex(key: &[u8], message: &[u8]) -> String {
    let mut hmac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    hmac.update(message);
    hex::encode(hmac.finalize().into_bytes())
}

const ENCODING_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'!')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

fn generate_canonical_query_string(params: &[(&str, &str)]) -> String {
    let mut encoded_params: Vec<String> = params
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                percent_encode(key.as_bytes(), ENCODING_SET),
                percent_encode(value.as_bytes(), ENCODING_SET)
            )
        })
        .collect();
    encoded_params.sort();
    encoded_params.join("&")
}
