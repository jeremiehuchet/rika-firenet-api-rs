use reqwest::Client;
use rika_firenet_client::{
    HasDetailledStatus, RikaFirenet, RikaFirenetClient,
    model::{DailySchedule, HeatPeriod, HeatingSchedule, StatusDetail},
};
use testcontainers::{
    ContainerAsync, GenericImage,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
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

trait RikaFirenetHost {
    fn rika_firenet_base_url(&self) -> impl Future<Output = String>;
    fn assert_mock_count(&self, feature: &str, expected_count: u32) -> impl Future<Output = ()>;
}

impl RikaFirenetHost for ContainerAsync<GenericImage> {
    async fn rika_firenet_base_url(&self) -> String {
        let listening_port = self
            .ports()
            .await
            .unwrap()
            .map_to_host_port_ipv4(3000)
            .unwrap();
        format!("http://127.0.0.1:{listening_port}")
    }

    async fn assert_mock_count(&self, feature: &str, expected_count: u32) {
        let base_url = self.rika_firenet_base_url().await;
        let http_client = Client::builder().build().expect("an http client");
        let request = http_client
            .get(format!("{base_url}/mock/{feature}"))
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
}
#[tokio::test]
async fn should_sucessfully_auto_login() {
    let container = start_rika_mock().await;
    let client = RikaFirenetClient::builder()
        .base_url(container.rika_firenet_base_url().await)
        .build("registered-user@rika-firenet.com", "Secret");

    client.list_stoves().await.unwrap();
}

#[tokio::test]
async fn can_list_stoves() {
    let container = start_rika_mock().await;
    let client = RikaFirenetClient::builder()
        .base_url(container.rika_firenet_base_url().await)
        .build("registered-user@rika-firenet.com", "Secret");

    let stoves = client.list_stoves().await.unwrap();
    assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");
}

#[tokio::test]
async fn can_list_stoves_multiple_times_with_one_single_authentication() {
    let container = start_rika_mock().await;
    let client = RikaFirenetClient::builder()
        .base_url(container.rika_firenet_base_url().await)
        .build("registered-user@rika-firenet.com", "Secret");

    let stoves = client.list_stoves().await.unwrap();
    assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");
    let stoves = client.list_stoves().await.unwrap();
    assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");
    let stoves = client.list_stoves().await.unwrap();
    assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");

    container.assert_mock_count("login-count", 1).await;
}

#[tokio::test]
async fn cant_list_stoves_with_invalid_credentials() {
    let container = start_rika_mock().await;
    let client = RikaFirenetClient::builder()
        .base_url(container.rika_firenet_base_url().await)
        .build("unknown-user@rika-firenet.com", "InvalidSecret");

    let empty_response = client.list_stoves().await.unwrap();
    assert_eq!(empty_response.len(), 0, "expect empty stoves ids");
}

#[tokio::test]
async fn can_get_stove_status() {
    let container = start_rika_mock().await;
    let client = RikaFirenetClient::builder()
        .base_url(container.rika_firenet_base_url().await)
        .build("registered-user@rika-firenet.com", "Secret");

    let stove = client.status("12345").await.unwrap();

    assert_eq!(stove.stove_id, "12345");
    assert_eq!(stove.oem, "RIKA");
    assert_eq!(stove.name, "Stove 12345");
    assert_eq!(stove.sensors.input_room_temperature, "19.6");
}

#[tokio::test]
async fn can_log_out() {
    let container = start_rika_mock().await;
    container.assert_mock_count("logout-count", 0).await;

    let client = RikaFirenetClient::builder()
        .base_url(container.rika_firenet_base_url().await)
        .build("registered-user@rika-firenet.com", "Secret");
    client.list_stoves().await.unwrap();
    client.logout().await.unwrap();

    container.assert_mock_count("logout-count", 1).await;

    client.list_stoves().await.unwrap();
    client.logout().await.unwrap();

    container.assert_mock_count("logout-count", 2).await;
}

#[tokio::test]
async fn can_turn_stove_off_and_on() {
    let container = start_rika_mock().await;
    let client = RikaFirenetClient::builder()
        .base_url(container.rika_firenet_base_url().await)
        .build("registered-user@rika-firenet.com", "Secret");

    let stove = client.status("12345").await.unwrap();
    assert_eq!(stove.controls.on_off, Some(true), "stove control is on");
    assert_eq!(
        stove.get_status_details(),
        StatusDetail::Standby,
        "stove status is standby"
    );

    client.turn_off("12345").await.unwrap();

    let stove = client.status("12345").await.unwrap();
    assert_eq!(stove.controls.on_off, Some(false), "stove control is off");
    assert_eq!(
        stove.get_status_details(),
        StatusDetail::Off,
        "stove status is off"
    );

    client.turn_on("12345").await.unwrap();

    let stove = client.status("12345").await.unwrap();
    assert_eq!(stove.controls.on_off, Some(true), "stove is on");
    assert_eq!(
        stove.get_status_details(),
        StatusDetail::Standby,
        "stove status is standby"
    );
}

#[tokio::test]
async fn can_execute_sample_senario() {
    let container = start_rika_mock().await;

    let stove_id = "12345";
    let client = RikaFirenetClient::builder()
        .base_url(container.rika_firenet_base_url().await)
        .build("registered-user@rika-firenet.com", "Secret");

    // let stove_id = "12345";
    // let client = RikaFirenetClient::builder()
    //     .base_url("https://www.rika-firenet.com".to_string())
    //     .build("registered-user@rika-firenet.com", "Secret")
    //    ;

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
        .restore_controls(stove_id, original_status.controls.clone())
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
