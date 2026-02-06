use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use common::models::TimeData;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health,
        handlers::get_time,
    ),
    components(schemas(
        TimeData,
    )),
    tags(
        (name = "time", description = "Time data endpoints"),
    ),
)]
struct ApiDoc;

pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi())
}

