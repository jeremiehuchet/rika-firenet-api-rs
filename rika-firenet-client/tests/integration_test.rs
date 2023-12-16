use reqwest::Client;
use rika_firenet_client::{RikaFirenetClient, RikaFirenetClientBuilder};
use testcontainers::{
    clients::{self},
    core::WaitFor,
    Container, Image,
};

#[derive(Default)]
struct RikaMock {}

impl Image for RikaMock {
    type Args = Vec<String>;

    fn name(&self) -> String {
        String::from("rika-firenet-api-mock")
    }

    fn tag(&self) -> String {
        String::from("latest")
    }
    fn expose_ports(&self) -> Vec<u16> {
        vec![3000]
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::StdOutMessage {
            message: String::from("Rika Firenet mock listening on port 3000"),
        }]
    }
}

fn client_for<'d>(container: &Container<'d, RikaMock>) -> RikaFirenetClientBuilder {
    let listening_port = container.ports().map_to_host_port_ipv4(3000).unwrap();
    RikaFirenetClient::builder().base_url(format!("http://127.0.0.1:{listening_port}",))
}

async fn assert_mock_count<'d>(
    feature: &str,
    expected_count: u32,
    container: &Container<'d, RikaMock>,
) {
    let listening_port = container.ports().map_to_host_port_ipv4(3000).unwrap();
    let http_client = Client::builder().build().expect("an http client");
    let request = http_client
        .get(format!("http://127.0.0.1:{listening_port}/mock/{feature}"))
        .build()
        .unwrap();
    let actual_count: u32 = http_client
        .execute(request)
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
        .as_str()
        .parse()
        .unwrap();
    assert_eq!(actual_count, expected_count, "expecting {expected_count}");
}

#[tokio::test]
async fn should_sucessfully_auto_login() {
    let docker = clients::Cli::default();
    let container = docker.run(RikaMock::default());
    let client = client_for(&container)
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();

    client.list_stoves().await.unwrap();
}

#[tokio::test]
async fn can_list_stoves() {
    let docker = clients::Cli::default();
    let container = docker.run(RikaMock::default());
    let client = client_for(&container)
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();

    let stoves = client.list_stoves().await.unwrap();
    assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");
}

#[tokio::test]
async fn can_list_stoves_multiple_times_with_one_single_authentication() {
    let docker = clients::Cli::default();
    let container = docker.run(RikaMock::default());
    let client = client_for(&container)
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();

    let stoves = client.list_stoves().await.unwrap();
    assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");
    let stoves = client.list_stoves().await.unwrap();
    assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");
    let stoves = client.list_stoves().await.unwrap();
    assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");

    assert_mock_count("login-count", 1, &container).await;
}

#[tokio::test]
async fn cant_list_stoves_with_invalid_credentials() {
    let docker = clients::Cli::default();
    let container = docker.run(RikaMock::default());
    let client = client_for(&container)
        .credentials("unknown-user@rika-firenet.com", "InvalidSecret")
        .build();

    let empty_response = client.list_stoves().await.unwrap();
    assert_eq!(empty_response.len(), 0, "expect empty stoves ids");
}

#[tokio::test]
async fn can_get_stove_status() {
    let docker = clients::Cli::default();
    let container = docker.run(RikaMock::default());
    let client = client_for(&container)
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();

    let stove = client.status("12345".to_string()).await.unwrap();

    assert_eq!(stove.stove_id, "12345");
    assert_eq!(stove.oem, "RIKA");
    assert_eq!(stove.name, "Stove 12345");
    assert_eq!(stove.sensors.input_room_temperature, "19.6");
}

#[tokio::test]
async fn can_log_out() {
    let docker = clients::Cli::default();
    let container = docker.run(RikaMock::default());
    assert_mock_count("logout-count", 0, &container).await;

    let client = client_for(&container)
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();
    client.list_stoves().await.unwrap();
    client.logout().await.unwrap();

    assert_mock_count("logout-count", 1, &container).await;

    client.list_stoves().await.unwrap();
    client.logout().await.unwrap();

    assert_mock_count("logout-count", 2, &container).await;
}
