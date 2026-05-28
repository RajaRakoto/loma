//! Axum HTTP API server.
//!
//! # Routes
//! | Method | Path         | Description          |
//! |--------|--------------|----------------------|
//! | GET    | /            | Welcome message      |
//! | GET    | /health      | Health check         |
//! | GET    | /users       | List users (example) |
//! | GET    | /users/:id   | Get user by ID       |
//! | POST   | /users       | Create a user        |
//! | PUT    | /users/:id   | Update a user        |
//! | DELETE | /users/:id   | Delete a user        |

use crate::error::{Error, Result};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{info, warn};
use uuid::Uuid;

// ── Response types ─────────────────────────────────────────────────────────────

/// Generic success / error envelope returned by every endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub timestamp: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now().to_rfc3339(),
        })
    }

    pub fn err(message: impl Into<String>) -> Json<Self> {
        Json(Self {
            success: false,
            data: None,
            error: Some(message.into()),
            timestamp: Utc::now().to_rfc3339(),
        })
    }
}

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

/// Example user resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

/// Request body for creating a user.
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

/// Request body for updating a user (all fields optional).
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Query parameters for the user list endpoint.
#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search: Option<String>,
}

// ── Server bootstrap ──────────────────────────────────────────────────────────

/// Build and start the Axum server on `port`.
pub async fn start_server(port: u16) -> Result<()> {
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let addr = format!("{}:{}", host, port);

    let app = build_router();

    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| Error::api(format!("Cannot bind to {addr}: {e}")))?;

    info!("API server listening on http://{addr}");
    info!("  GET  /           — welcome");
    info!("  GET  /health     — health check");
    info!("  GET  /users      — list users");
    info!("  POST /users      — create user");
    info!("  GET  /users/:id  — get user");
    info!("  PUT  /users/:id  — update user");
    info!("  DEL  /users/:id  — delete user");

    axum::serve(listener, app)
        .await
        .map_err(|e| Error::api(format!("Server error: {e}")))?;

    Ok(())
}

/// Construct the [`Router`] (also useful in integration tests).
pub fn build_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/:id",
            get(get_user).put(update_user).delete(delete_user),
        )
        .layer(middleware::from_fn(log_requests))
}

// ── Middleware ────────────────────────────────────────────────────────────────

async fn log_requests(req: axum::extract::Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let response = next.run(req).await;
    info!("{} {} → {}", method, uri, response.status());
    response
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn root() -> Json<ApiResponse<String>> {
    ApiResponse::ok("Welcome to Claude Code Manager API".to_string())
}

async fn health() -> Json<ApiResponse<HealthResponse>> {
    ApiResponse::ok(HealthResponse {
        status: "healthy".into(),
        version: crate::VERSION.into(),
        timestamp: Utc::now().to_rfc3339(),
    })
}

/// In-memory stub — replace with real DB queries in your project.
fn stub_users() -> Vec<User> {
    vec![
        User {
            id: "1".into(),
            name: "Alice Martin".into(),
            email: "alice@example.com".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
        },
        User {
            id: "2".into(),
            name: "Bob Dupont".into(),
            email: "bob@example.com".into(),
            created_at: "2024-01-02T00:00:00Z".into(),
        },
    ]
}

async fn list_users(Query(params): Query<UserListQuery>) -> Json<ApiResponse<Vec<User>>> {
    let mut users = stub_users();

    // Optional search filter
    if let Some(ref search) = params.search {
        let s = search.to_lowercase();
        users.retain(|u| u.name.to_lowercase().contains(&s) || u.email.to_lowercase().contains(&s));
    }

    // Pagination
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(10);
    let page: Vec<User> = users.into_iter().skip(offset).take(limit).collect();

    ApiResponse::ok(page)
}

async fn get_user(Path(id): Path<String>) -> impl IntoResponse {
    match stub_users().into_iter().find(|u| u.id == id) {
        Some(user) => ApiResponse::ok(user).into_response(),
        None => {
            warn!("User not found: {id}");
            (
                StatusCode::NOT_FOUND,
                ApiResponse::<()>::err("User not found"),
            )
                .into_response()
        }
    }
}

async fn create_user(Json(body): Json<CreateUserRequest>) -> impl IntoResponse {
    // Basic validation
    if body.name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiResponse::<()>::err("name must not be empty"),
        )
            .into_response();
    }
    if !body.email.contains('@') {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiResponse::<()>::err("email is invalid"),
        )
            .into_response();
    }

    let user = User {
        id: Uuid::new_v4().to_string(),
        name: body.name,
        email: body.email,
        created_at: Utc::now().to_rfc3339(),
    };

    (StatusCode::CREATED, ApiResponse::ok(user)).into_response()
}

async fn update_user(
    Path(id): Path<String>,
    Json(body): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    match stub_users().into_iter().find(|u| u.id == id) {
        Some(mut user) => {
            if let Some(name) = body.name {
                user.name = name;
            }
            if let Some(email) = body.email {
                user.email = email;
            }
            ApiResponse::ok(user).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            ApiResponse::<()>::err("User not found"),
        )
            .into_response(),
    }
}

async fn delete_user(Path(id): Path<String>) -> impl IntoResponse {
    match stub_users().into_iter().find(|u| u.id == id) {
        Some(_) => ApiResponse::ok(format!("User {id} deleted")).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            ApiResponse::<()>::err("User not found"),
        )
            .into_response(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn health_returns_200() {
        let app = build_router();
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn list_users_returns_200() {
        let app = build_router();
        let req = Request::builder()
            .uri("/users")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_unknown_user_returns_404() {
        let app = build_router();
        let req = Request::builder()
            .uri("/users/99999")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn create_user_bad_email_returns_422() {
        let app = build_router();
        let body = serde_json::json!({ "name": "Test", "email": "not-an-email" });
        let req = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}