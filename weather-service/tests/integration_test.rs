use serde_json::json;
use wiremock::{
    matchers::{method, path, path_regex},
    Mock, MockServer, ResponseTemplate,
};

/// Test that the mock server can serve weather-like responses
#[tokio::test]
async fn test_mock_weather_api() {
    let mock_server = MockServer::start().await;

    // Mock Open-Meteo API response
    Mock::given(method("GET"))
        .and(path_regex(r"/v1/forecast.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "current": {
                "temperature_2m": 15.5,
                "relative_humidity_2m": 65.0,
                "wind_speed_10m": 10.2,
                "weather_code": 3
            }
        })))
        .mount(&mock_server)
        .await;

    // Make a request to the mock server
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/v1/forecast?latitude=51.5&longitude=-0.1&current=temperature_2m",
            mock_server.uri()
        ))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["current"]["temperature_2m"], 15.5);
}

/// Test that the mock server can serve time-like responses
#[tokio::test]
async fn test_mock_time_api() {
    let mock_server = MockServer::start().await;

    // Mock WorldTimeAPI response
    Mock::given(method("GET"))
        .and(path("/api/time/London"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "datetime": "2024-01-01T12:00:00+00:00",
            "timezone": "Europe/London",
            "unix_time": 1704110400
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/time/London", mock_server.uri()))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["timezone"], "Europe/London");
}

/// Test error handling with timeout simulation
#[tokio::test]
async fn test_timeout_handling() {
    let mock_server = MockServer::start().await;

    // Mock a slow response that would timeout
    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(5)))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(100))
        .build()
        .expect("Failed to build client");

    let result = client
        .get(format!("{}/slow", mock_server.uri()))
        .send()
        .await;

    // Should timeout
    assert!(result.is_err());
}

/// Test HTTP error responses
#[tokio::test]
async fn test_http_error_responses() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/error", mock_server.uri()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status().as_u16(), 500);
}
