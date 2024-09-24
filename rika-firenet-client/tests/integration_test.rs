use reqwest::Client;
use rika_firenet_client::{
    model::{DailySchedule, HeatPeriod, HeatingSchedule},
    RikaFirenetClient, RikaFirenetClientBuilder,
};
use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage,
};

async fn start_rika_mock() -> ContainerAsync<GenericImage> {
    GenericImage::new("rika-firenet-api-mock", "latest")
        .with_exposed_port(ContainerPort::Tcp(3000))
        .with_wait_for(WaitFor::message_on_stdout(
            "Rika Firenet mock listening on port 3000",
        ))
        .start()
        .await
        .unwrap()
}

async fn client_for<'d>(container: &ContainerAsync<GenericImage>) -> RikaFirenetClientBuilder {
    let listening_port = container
        .ports()
        .await
        .unwrap()
        .map_to_host_port_ipv4(3000)
        .unwrap();
    RikaFirenetClient::builder().base_url(format!("http://127.0.0.1:{listening_port}",))
}

async fn assert_mock_count(
    feature: &str,
    expected_count: u32,
    container: &ContainerAsync<GenericImage>,
) {
    let listening_port = container
        .ports()
        .await
        .unwrap()
        .map_to_host_port_ipv4(3000)
        .unwrap();
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
    let container = start_rika_mock().await;
    let client = client_for(&container)
        .await
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();

    client.list_stoves().await.unwrap();
}

#[tokio::test]
async fn can_list_stoves() {
    let container = start_rika_mock().await;
    let client = client_for(&container)
        .await
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();

    let stoves = client.list_stoves().await.unwrap();
    assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");
}

#[tokio::test]
async fn can_list_stoves_multiple_times_with_one_single_authentication() {
    let container = start_rika_mock().await;
    let client = client_for(&container)
        .await
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
    let container = start_rika_mock().await;
    let client = client_for(&container)
        .await
        .credentials("unknown-user@rika-firenet.com", "InvalidSecret")
        .build();

    let empty_response = client.list_stoves().await.unwrap();
    assert_eq!(empty_response.len(), 0, "expect empty stoves ids");
}

#[tokio::test]
async fn can_get_stove_status() {
    let container = start_rika_mock().await;
    let client = client_for(&container)
        .await
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();

    let stove = client.status("12345").await.unwrap();

    assert_eq!(stove.stove_id, "12345");
    assert_eq!(stove.oem, "RIKA");
    assert_eq!(stove.name, "Stove 12345");
    assert_eq!(stove.sensors.input_room_temperature, "19.6");
}

#[tokio::test]
async fn can_log_out() {
    let container = start_rika_mock().await;
    assert_mock_count("logout-count", 0, &container).await;

    let client = client_for(&container)
        .await
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();
    client.list_stoves().await.unwrap();
    client.logout().await.unwrap();

    assert_mock_count("logout-count", 1, &container).await;

    client.list_stoves().await.unwrap();
    client.logout().await.unwrap();

    assert_mock_count("logout-count", 2, &container).await;
}

#[tokio::test]
async fn can_turn_stove_off_and_on() {
    let container = start_rika_mock().await;
    let client = client_for(&container)
        .await
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();

    let stove = client.status("12345").await.unwrap();
    assert_eq!(stove.controls.on_off, Some(true), "stove is on");

    client.turn_off("12345").await.unwrap();

    let stove = client.status("12345").await.unwrap();
    assert_eq!(stove.controls.on_off, Some(false), "stove is off");

    client.turn_on("12345").await.unwrap();

    let stove = client.status("12345").await.unwrap();
    assert_eq!(stove.controls.on_off, Some(true), "stove is on");
}

#[tokio::test]
async fn can_execute_sample_senario() {
    let container = start_rika_mock().await;

    let stove_id = "12345";
    let client = client_for(&container)
        .await
        .credentials("registered-user@rika-firenet.com", "Secret")
        .build();

    // let stove_id = "12345";
    // let client = RikaFirenetClient::builder()
    //     .base_url("https://www.rika-firenet.com".to_string())
    //     .credentials("registered-user@rika-firenet.com", "Secret")
    //     .build();

    let original_status = client.status(stove_id).await.unwrap();
    println!("\nstove status:\n{original_status:?}");

    client.turn_off(stove_id).await.unwrap();
    let status = client.status(stove_id).await.unwrap();
    println!("\nstove status:\n{status:?}");
    assert_eq!(status.controls.on_off, Some(false), "stove off");

    client.set_manual_mode(stove_id, 30).await.unwrap();
    let status = client.status(stove_id).await.unwrap();
    println!("\nstove status:\n{status:?}");
    assert_eq!(status.controls.operating_mode, Some(0), "manual mode");

    let schedule = HeatingSchedule::week_vs_end_days(
        DailySchedule::dual(
            HeatPeriod::new(7, 30, 10, 00).unwrap(),
            HeatPeriod::new(18, 15, 22, 45).unwrap(),
        ),
        DailySchedule::single(HeatPeriod::new(10, 15, 23, 00).unwrap()),
    );
    client.enable_schedule(stove_id, schedule).await.unwrap();
    let status = client.status(stove_id).await.unwrap();
    println!("\nstove status:\n{status:?}");
    assert_eq!(
        status.controls.heating_times_active_for_comfort,
        Some(true),
        "heating schedule on"
    );
    assert_eq!(
        status.controls.heating_time_mon1,
        Some("07301000".to_string()),
        "monday am"
    );
    assert_eq!(
        status.controls.heating_time_mon2,
        Some("18152245".to_string()),
        "monday pm"
    );
    assert_eq!(
        status.controls.heating_time_tue1,
        Some("07301000".to_string()),
        "tuesday am"
    );
    assert_eq!(
        status.controls.heating_time_tue2,
        Some("18152245".to_string()),
        "tuesday pm"
    );
    assert_eq!(
        status.controls.heating_time_wed1,
        Some("07301000".to_string()),
        "wednesday am"
    );
    assert_eq!(
        status.controls.heating_time_wed2,
        Some("18152245".to_string()),
        "wednesday pm"
    );
    assert_eq!(
        status.controls.heating_time_thu1,
        Some("07301000".to_string()),
        "thuesday am"
    );
    assert_eq!(
        status.controls.heating_time_thu2,
        Some("18152245".to_string()),
        "thuesday pm"
    );
    assert_eq!(
        status.controls.heating_time_fri1,
        Some("07301000".to_string()),
        "friday am"
    );
    assert_eq!(
        status.controls.heating_time_fri2,
        Some("18152245".to_string()),
        "friday pm"
    );
    assert_eq!(
        status.controls.heating_time_sat1,
        Some("10152300".to_string()),
        "saturday am"
    );
    assert_eq!(
        status.controls.heating_time_sat2,
        Some("00000000".to_string()),
        "saturday pm"
    );
    assert_eq!(
        status.controls.heating_time_sun1,
        Some("10152300".to_string()),
        "sunday am"
    );
    assert_eq!(
        status.controls.heating_time_sun2,
        Some("00000000".to_string()),
        "sunday pm"
    );

    client.set_comfort_mode(stove_id, 18, 20).await.unwrap();
    let status = client.status(stove_id).await.unwrap();
    println!("\nstove status:\n{status:?}");
    assert_eq!(status.controls.operating_mode, Some(2), "comfort mode");
    assert_eq!(
        status.controls.set_back_temperature,
        Some("18".to_string()),
        "target temperature"
    );
    assert_eq!(
        status.controls.target_temperature,
        Some("20".to_string()),
        "target temperature"
    );

    client
        .restore_controls(stove_id, original_status.controls.as_ref().clone())
        .await
        .unwrap();
    let status = client.status(stove_id).await.unwrap();
    println!("\nstove status:\n{status:?}");

    client.turn_on(stove_id).await.unwrap();
    let status = client.status(stove_id).await.unwrap();
    println!("\nstove status:\n{status:?}");
    assert_eq!(status.controls.on_off, Some(true), "stove on");

    client.logout().await.unwrap();
}
