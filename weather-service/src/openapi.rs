use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use common::models::{AggregateResponse, WeatherData};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health,
        handlers::get_weather,
        handlers::aggregate,
    ),
    components(schemas(
        WeatherData,
        AggregateResponse,
        common::models::CityData,
        common::models::TimeData,
        common::models::ResponseSummary,
    )),
    tags(
        (name = "weather", description = "Weather data endpoints"),
        (name = "aggregate", description = "City data aggregation"),
    ),
)]
struct ApiDoc;

pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi())
}
