use crate::{db::DbPool, error::ApiError, queries};
use runelink_types::User;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Session {
    pub user: User,
    pub is_admin: bool, // TODO: this will be inside the User struct
}

#[derive(Clone, Debug)]
pub struct AuthBuilder {
    user_id: Option<Uuid>, // TODO: replace with tokens/auth info
    admin: bool,
    server_member: Option<Uuid>,
    server_admin: Option<Uuid>,
    expected_user_id: Option<Uuid>,
    server_admin_override: Option<Uuid>,
}

impl AuthBuilder {
    pub fn new(user_id: Option<Uuid>) -> Self {
        AuthBuilder {
            user_id,
            admin: false,
            server_member: None,
            server_admin: None,
            expected_user_id: None,
            server_admin_override: None,
        }
    }

    #[allow(dead_code)]
    pub fn admin(mut self) -> Self {
        self.admin = true;
        self
    }

    #[allow(dead_code)]
    pub fn user(mut self, user_id: Uuid) -> Self {
        self.expected_user_id = Some(user_id);
        self
    }

    #[allow(dead_code)]
    pub fn server_member(mut self, server_id: Uuid) -> Self {
        self.server_member = Some(server_id);
        self
    }

    #[allow(dead_code)]
    pub fn server_admin(mut self, server_id: Uuid) -> Self {
        self.server_admin = Some(server_id);
        self
    }

    #[allow(dead_code)]
    pub fn or_server_admin(mut self, server_id: Uuid) -> Self {
        self.server_admin_override = Some(server_id);
        self
    }

    pub async fn build(&self, pool: &DbPool) -> Result<Session, ApiError> {
        let Some(user_id) = self.user_id else {
            return Err(ApiError::AuthError("No credentials provided".into()));
        };
        let user = queries::get_user_by_id(pool, user_id)
            .await
            .map_err(|_| ApiError::AuthError("Invalid credentials".into()))?;

        let user_is_admin = true; // for testing only

        // TODO: early success return for host admins

        // TODO: early success return if admin in server_admin_override

        if self.admin && !user_is_admin { // redundant check?
            return Err(ApiError::AuthError("Admin only".into()));
        }

        // TODO: check required server member and admin

        Ok(Session {
            user,
            is_admin: user_is_admin,
        })
    }

    #[allow(dead_code)]
    pub async fn build_optional(
        self, _pool: &DbPool
    ) -> Result<Option<Session>, ApiError> {
        // allow guests but fetch user info if they are logged in
        todo!()
    }
}
