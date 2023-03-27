use std::fmt::Error;

use httpmock::{
    Method::{GET, POST},
    MockServer, Regex,
};

use rika_firenet_client::RikaFirenetClient;

#[tokio::test]
async fn can_login() {
    let server = MockServer::start();
    let login_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/web/login")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body_matches(
                Regex::new("^(email=[^&]*&password=.*|password=[^&]*&email=.*)$").unwrap(),
            );
        then.status(302).header(
            "Set-Cookie",
            "connect.sid=xxx.xxx; Path=/; Expires=Fri, 10 Mar 2063 15:14:41 GMT; HttpOnly",
        );
    });

    let client = RikaFirenetClient::new_with_base_url(server.base_url());

    client
        .login("someone@rika.com".to_string(), "Secret!".to_string())
        .await
        .unwrap();
    login_mock.assert();
}

#[tokio::test]
async fn should_forward_cookie() {
    let server = MockServer::start();
    let login_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/web/login")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body_matches(
                Regex::new("^(email=[^&]*&password=.*|password=[^&]*&email=.*)$").unwrap(),
            );
        then.status(302).header(
            "Set-Cookie",
            "connect.sid=xxx.xxx; Path=/; Expires=Fri, 10 Mar 2063 15:14:41 GMT; HttpOnly",
        );
    });
    let logout_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/web/logout")
            .header("Cookie", "connect.sid=xxx.xxx");
        then.status(302).header(
            "Set-Cookie",
            "connect.sid=xxx.xxx; Path=/; Expires=Fri, 10 Mar 2063 15:14:41 GMT; HttpOnly",
        );
    });

    let client = RikaFirenetClient::new_with_base_url(server.base_url());

    client
        .login("someone@rika.com".to_string(), "Secret!".to_string())
        .await
        .unwrap();
    login_mock.assert();

    client.logout().await.unwrap();
    logout_mock.assert();
}

#[tokio::test]
async fn can_extract_stove_list() {
    let server = MockServer::start();
    let summary_mock = server.mock(|when, then| {
        when.method(GET).path("/web/summary");
        then.status(200)
            .body_from_file("tests/resources/rika_firenet_web_summary.html");
    });

    let client = RikaFirenetClient::new_with_base_url(server.base_url());

    let stoves = client.list_stoves().await.unwrap();

    summary_mock.assert();
    assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");
}

#[tokio::test]
async fn can_get_stove_status() {
    let server = MockServer::start();
    let status_mock = server.mock(|when, then| {
        when.method(GET).path("/api/client/12345/status");
        then.status(200)
            .body_from_file("tests/resources/rika_firenet_api_status.json");
    });

    let client = RikaFirenetClient::new_with_base_url(server.base_url());

    let status = client.status("12345".to_string()).await.unwrap();

    status_mock.assert();
    assert_eq!(status.stove_id, "12345", "stove id");
    assert_eq!(status.name, "Stove n1", "stove name");
    assert_eq!(
        status.sensors.input_room_temperature, "19.6",
        "sensor value"
    );
}
