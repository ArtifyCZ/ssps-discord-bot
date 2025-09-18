use application_ports::role_sync_job_handler::{RoleSyncJobHandlerError, RoleSyncJobHandlerPort};
use async_trait::async_trait;
use domain::authentication::authenticated_user::{
    AuthenticatedUser, AuthenticatedUserRepository, AuthenticatedUserRepositoryError,
};
use domain::authentication::create_class_ids;
use domain::jobs::role_sync_job::{
    RoleSyncRequested, RoleSyncRequestedRepository, RoleSyncRequestedRepositoryError,
};
use domain::ports::discord::{DiscordError, DiscordPort, Role, RoleDiff};
use domain_shared::discord::RoleId;
use std::ops::Not;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, instrument};

pub struct RoleSyncJobHandler {
    discord_port: Arc<dyn DiscordPort + Send + Sync>,
    authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
    role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
    additional_student_roles: Vec<RoleId>,
    class_ids: Vec<String>,
    class_id_to_role_id: Mutex<Option<Vec<(String, RoleId)>>>,
}

impl RoleSyncJobHandler {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        discord_port: Arc<dyn DiscordPort + Send + Sync>,
        authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
        role_sync_requested_repository: Arc<dyn RoleSyncRequestedRepository + Send + Sync>,
        additional_student_roles: Vec<RoleId>,
    ) -> Self {
        let class_ids = create_class_ids();
        let class_id_to_role_id = Mutex::new(None);

        Self {
            discord_port,
            authenticated_user_repository,
            role_sync_requested_repository,
            additional_student_roles,
            class_ids,
            class_id_to_role_id,
        }
    }

    #[instrument(level = "info", skip(self))]
    async fn handle(&self, request: RoleSyncRequested) -> Result<(), RoleSyncJobHandlerError> {
        let (class_id_to_role_id, assigned_roles, user) = tokio::try_join!(
            async move { self.get_or_create_class_id_to_role_id().await },
            async move {
                self.discord_port
                    .find_user_roles(request.user_id)
                    .await
                    .map_err(map_discord_err)
            },
            async move {
                self.authenticated_user_repository
                    .find_by_user_id(request.user_id)
                    .await
                    .map_err(map_user_repo_err)
            }
        )?;

        let role_diff = match user {
            None => self.handle_unauthenticated_user(&assigned_roles),
            Some(user) => {
                self.handle_authenticated_user(&user, &assigned_roles, &class_id_to_role_id)
            }
        }?;

        self.discord_port
            .apply_role_diff(request.user_id, &role_diff, "Role sync job handler")
            .await
            .map_err(map_discord_err)?;

        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    fn handle_unauthenticated_user(
        &self,
        assigned_roles: &Vec<Role>,
    ) -> Result<RoleDiff, RoleSyncJobHandlerError> {
        let mut to_remove = Vec::new();

        for additional_student_role in &self.additional_student_roles {
            if assigned_roles
                .iter()
                .any(|r| r.role_id == *additional_student_role)
            {
                to_remove.push(*additional_student_role);
            }
        }

        for assigned_role in assigned_roles {
            for class_id in &self.class_ids {
                if assigned_role.name.eq_ignore_ascii_case(class_id) {
                    to_remove.push(assigned_role.role_id);
                }
            }
        }

        Ok(RoleDiff {
            to_assign: vec![],
            to_remove,
        })
    }

    #[instrument(level = "trace", skip(self))]
    fn handle_authenticated_user(
        &self,
        user: &AuthenticatedUser,
        assigned_roles: &Vec<Role>,
        class_id_to_role_id: &Vec<(String, RoleId)>,
    ) -> Result<RoleDiff, RoleSyncJobHandlerError> {
        let mut to_assign = Vec::new();
        let mut to_remove = Vec::new();

        for additional_student_role in &self.additional_student_roles {
            if assigned_roles
                .iter()
                .any(|r| r.role_id == *additional_student_role)
                .not()
            {
                to_assign.push(*additional_student_role);
            }
        }

        // The user has a class role that does not match their class_id
        for assigned_role in assigned_roles {
            for class_id in &self.class_ids {
                if class_id != user.class_id() && assigned_role.name.eq_ignore_ascii_case(class_id)
                {
                    to_remove.push(assigned_role.role_id);
                }
            }
        }

        let (_, class_role_id) = class_id_to_role_id
            .iter()
            .find(|(c, _)| c.eq_ignore_ascii_case(user.class_id()))
            .ok_or_else(|| {
                error!(
                    user_id = user.user_id().0,
                    "Class role not found for user's class {}",
                    user.class_id(),
                );
                RoleSyncJobHandlerError::TemporaryUnavailable
            })?;

        if assigned_roles
            .iter()
            .any(|r| r.role_id == *class_role_id)
            .not()
        {
            to_assign.push(*class_role_id);
        }

        Ok(RoleDiff {
            to_assign,
            to_remove,
        })
    }

    #[instrument(level = "trace", skip(self))]
    async fn get_or_create_class_id_to_role_id(
        &self,
    ) -> Result<Vec<(String, RoleId)>, RoleSyncJobHandlerError> {
        let mut class_id_to_role_id_guard = self.class_id_to_role_id.lock().await;
        if let Some(class_id_to_role_id) = &*class_id_to_role_id_guard {
            return Ok(class_id_to_role_id.clone());
        }
        let mut class_id_to_role_id = Vec::new();

        for class_id in &self.class_ids {
            let role = self
                .discord_port
                .find_or_create_role_by_name(&class_id.to_uppercase(), "Role for students of class")
                .await
                .map_err(map_discord_err)?;
            class_id_to_role_id.push((class_id.to_string(), role.role_id));
        }

        *class_id_to_role_id_guard = Some(class_id_to_role_id.clone());

        Ok(class_id_to_role_id)
    }
}

#[async_trait]
impl RoleSyncJobHandlerPort for RoleSyncJobHandler {
    #[instrument(level = "debug", skip_all)]
    async fn tick(&self) -> Result<(), RoleSyncJobHandlerError> {
        let high_priority = self
            .role_sync_requested_repository
            .pop_oldest(false)
            .await
            .map_err(map_sync_req_repo_err)?;

        if let Some(request) = high_priority {
            self.handle(request).await?;
            return Ok(());
        }

        let low_priority = self
            .role_sync_requested_repository
            .pop_oldest(true)
            .await
            .map_err(map_sync_req_repo_err)?;

        if let Some(request) = low_priority {
            self.handle(request).await?;
            return Ok(());
        }

        Err(RoleSyncJobHandlerError::NoRequestToHandle)
    }
}

#[instrument(level = "trace", skip_all)]
fn map_discord_err(err: DiscordError) -> RoleSyncJobHandlerError {
    match err {
        DiscordError::DiscordUnavailable => {
            error!("DiscordError::DiscordUnavailable");
            RoleSyncJobHandlerError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_user_repo_err(err: AuthenticatedUserRepositoryError) -> RoleSyncJobHandlerError {
    match err {
        AuthenticatedUserRepositoryError::ServiceUnavailable => {
            error!("AuthenticatedUserRepositoryError::ServiceUnavailable");
            RoleSyncJobHandlerError::TemporaryUnavailable
        }
    }
}

#[instrument(level = "trace", skip_all)]
fn map_sync_req_repo_err(err: RoleSyncRequestedRepositoryError) -> RoleSyncJobHandlerError {
    match err {
        RoleSyncRequestedRepositoryError::ServiceUnavailable => {
            error!("RoleSyncRequestedRepositoryError::ServiceUnavailable");
            RoleSyncJobHandlerError::TemporaryUnavailable
        }
    }
}
