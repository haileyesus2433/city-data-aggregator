use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use common::errors::AppError;
use common::models::{CreateUserRequest, LoginRequest, LoginResponse, UserResponse};
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::db::queries::User;
use crate::jwt::JwtService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service health check")
    )
)]
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "auth-service" }))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = User::find_by_username(&state.pool, &payload.username)
        .await
        .map_err(|e| AppError::database(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::auth("Invalid username or password"))?;

    let is_valid = bcrypt::verify(&payload.password, &user.password_hash)
        .map_err(|_| AppError::internal("Password verification failed"))?;

    if !is_valid {
        return Err(AppError::auth("Invalid username or password"));
    }

    let jwt_service = JwtService::new(state.jwt_secret.as_str());

    let permissions = User::get_permissions(&state.pool, &user.role)
        .await
        .map_err(|e| AppError::database(format!("Failed to get permissions: {}", e)))?;

    let token = jwt_service
        .generate_token(&user.id.to_string(), &user.role, permissions, 24)
        .map_err(|e| AppError::internal(format!("JWT generation failed: {}", e)))?;

    info!(user_id = %user.id, "User logged in successfully");

    Ok(Json(LoginResponse {
        token,
        user: UserResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            role: user.role,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        },
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User registered successfully", body = UserResponse),
        (status = 400, description = "Validation error")
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    if payload.username.is_empty() || payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::validation(
            "Username, email, and password are required",
        ));
    }

    let password_hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::internal(format!("Password hashing failed: {}", e)))?;

    let role = payload.role.unwrap_or_else(|| "user".to_string());

    let user = User::create(
        &state.pool,
        &payload.username,
        &payload.email,
        &password_hash,
        &role,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") {
            AppError::validation("Username or email already exists")
        } else {
            AppError::database(format!("Failed to create user: {}", e))
        }
    })?;

    info!(user_id = %user.id, "User registered successfully");

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/users",
    responses(
        (status = 200, description = "List of all users", body = Vec<UserResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "admin"
)]
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = User::list_all(&state.pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list users: {}", e)))?;

    let responses: Vec<UserResponse> = users
        .into_iter()
        .map(|u| UserResponse {
            id: u.id.to_string(),
            username: u.username,
            email: u.email,
            role: u.role,
            created_at: u.created_at.to_rfc3339(),
            updated_at: u.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(responses))
}

#[utoipa::path(
    post,
    path = "/api/admin/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "admin"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    if payload.username.is_empty() || payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::validation(
            "Username, email, and password are required",
        ));
    }

    let password_hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::internal(format!("Password hashing failed: {}", e)))?;

    let role = payload.role.unwrap_or_else(|| "user".to_string());

    let user = User::create(
        &state.pool,
        &payload.username,
        &payload.email,
        &password_hash,
        &role,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") {
            AppError::validation("Username or email already exists")
        } else {
            AppError::database(format!("Failed to create user: {}", e))
        }
    })?;

    info!(user_id = %user.id, "Admin created user");

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User details", body = UserResponse),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "admin"
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UserResponse>, AppError> {
    let user_id =
        Uuid::parse_str(&id).map_err(|_| AppError::validation("Invalid user ID format"))?;

    let user = User::find_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::database(format!("Failed to get user: {}", e)))?
        .ok_or_else(|| AppError::http(404, "User not found"))?;

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/admin/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "admin"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let user_id =
        Uuid::parse_str(&id).map_err(|_| AppError::validation("Invalid user ID format"))?;

    let deleted = User::delete(&state.pool, user_id)
        .await
        .map_err(|e| AppError::database(format!("Failed to delete user: {}", e)))?;

    if deleted {
        info!(user_id = %user_id, "User deleted");
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::http(404, "User not found"))
    }
}

#[utoipa::path(
    put,
    path = "/api/admin/users/{id}/role",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    request_body(
        content = inline(serde_json::Value),
        description = "JSON with 'role' field"
    ),
    responses(
        (status = 200, description = "User role updated", body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "admin"
)]
pub async fn update_user_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<UserResponse>, AppError> {
    let user_id =
        Uuid::parse_str(&id).map_err(|_| AppError::validation("Invalid user ID format"))?;

    let role = payload
        .get("role")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::validation("Role is required"))?;

    let updated = User::update_role(&state.pool, user_id, role)
        .await
        .map_err(|e| AppError::database(format!("Failed to update user role: {}", e)))?;

    if !updated {
        return Err(AppError::http(404, "User not found"));
    }

    let user = User::find_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::database(format!("Failed to get user: {}", e)))?
        .ok_or_else(|| AppError::http(404, "User not found"))?;

    info!(user_id = %user_id, new_role = %role, "User role updated");

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
    }))
}
