use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn create(
        pool: &PgPool,
        username: &str,
        email: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<Self, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, role)
            VALUES ($1, $2, $3, $4)
            RETURNING id, username, email, password_hash, role, created_at, updated_at
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_username(
        pool: &PgPool,
        username: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, role, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, role, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, role, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_role(pool: &PgPool, id: Uuid, role: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2
            "#,
        )
        .bind(role)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_permissions(pool: &PgPool, role: &str) -> Result<Vec<String>, sqlx::Error> {
        let permissions = sqlx::query_scalar::<_, String>(
            r#"
            SELECT p.name
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role = $1
            "#,
        )
        .bind(role)
        .fetch_all(pool)
        .await?;

        Ok(permissions)
    }
}
