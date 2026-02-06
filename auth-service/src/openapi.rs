use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use common::models::{CreateUserRequest, LoginRequest, LoginResponse, UserResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health,
        handlers::login,
        handlers::register,
        handlers::list_users,
        handlers::create_user,
        handlers::get_user,
        handlers::delete_user,
        handlers::update_user_role,
    ),
    components(schemas(
        LoginRequest,
        LoginResponse,
        CreateUserRequest,
        UserResponse,
    )),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "admin", description = "Admin user management endpoints"),
    ),
)]
struct ApiDoc;

pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi())
}
