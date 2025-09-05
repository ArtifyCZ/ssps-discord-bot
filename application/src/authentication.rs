use application_ports::authentication::{AuthenticationError, AuthenticationPort};
use application_ports::discord::InviteLink;
use async_trait::async_trait;
use domain::authentication::authenticated_user::{
    create_user_from_successful_authentication, AuthenticatedUserRepository,
};
use domain::authentication::user_authentication_request::{
    create_user_authentication_request, UserAuthenticationRequestRepository,
};
use domain::class::class_group::find_class_group;
use domain::class::class_id::get_class_id;
use domain::ports::discord::DiscordPort;
use domain::ports::oauth::OAuthPort;
use domain_shared::authentication::{AuthenticationLink, ClientCallbackToken, CsrfToken};
use domain_shared::discord::{RoleId, UserId};
use std::sync::Arc;
use tracing::{info, instrument, warn, Span};

pub struct AuthenticationService {
    discord_port: Arc<dyn DiscordPort + Send + Sync>,
    oauth_port: Arc<dyn OAuthPort + Send + Sync>,
    authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
    user_authentication_request_repository:
        Arc<dyn UserAuthenticationRequestRepository + Send + Sync>,
    invite_link: InviteLink,
    additional_student_roles: Vec<RoleId>,
}

impl AuthenticationService {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        discord_port: Arc<dyn DiscordPort + Send + Sync>,
        oauth_port: Arc<dyn OAuthPort + Send + Sync>,
        authenticated_user_repository: Arc<dyn AuthenticatedUserRepository + Send + Sync>,
        user_authentication_request_repository: Arc<
            dyn UserAuthenticationRequestRepository + Send + Sync,
        >,
        invite_link: InviteLink,
        additional_student_roles: Vec<RoleId>,
    ) -> Self {
        Self {
            discord_port,
            oauth_port,
            authenticated_user_repository,
            user_authentication_request_repository,
            invite_link,
            additional_student_roles,
        }
    }
}

#[async_trait]
impl AuthenticationPort for AuthenticationService {
    #[instrument(level = "info", skip(self))]
    async fn create_authentication_link(
        &self,
        user_id: UserId,
    ) -> Result<AuthenticationLink, AuthenticationError> {
        if let Some(_user) = self
            .authenticated_user_repository
            .find_by_user_id(user_id)
            .await?
        {
            info!(user_id = user_id.0, "The user tried to authenticate again");
            return Err(AuthenticationError::AlreadyAuthenticated);
        }

        let (link, csrf_token) = self.oauth_port.create_authentication_link().await?;

        let request = create_user_authentication_request(csrf_token, user_id);

        self.user_authentication_request_repository
            .save(&request)
            .await?;

        info!(user_id = user_id.0, "Authentication link created");

        Ok(link)
    }

    #[instrument(level = "info", skip(self, csrf_token, client_callback_token))]
    async fn confirm_authentication(
        &self,
        csrf_token: CsrfToken,
        client_callback_token: ClientCallbackToken,
    ) -> Result<InviteLink, AuthenticationError> {
        let request = match self
            .user_authentication_request_repository
            .find_by_csrf_token(&csrf_token)
            .await?
        {
            Some(request) => request,
            None => {
                warn!(
                    csrf_token = csrf_token.0,
                    "The user tried to authenticate with an invalid CSRF token",
                );
                return Err(AuthenticationError::AuthenticationRequestNotFound);
            }
        };
        let user_id = request.user_id();
        Span::current().record("user_id", user_id.0);

        let oauth_token = self
            .oauth_port
            .exchange_code_after_callback(client_callback_token)
            .await?;
        let groups = self
            .oauth_port
            .get_user_groups(&oauth_token.access_token)
            .await?;
        let class_group = find_class_group(&groups)
            .ok_or_else(|| AuthenticationError::Error("User is not in the Class group".into()))?;
        let class_id = get_class_id(class_group)
            .ok_or_else(|| AuthenticationError::Error("User's class group ID not found".into()))?;
        let user_info = self
            .oauth_port
            .get_user_info(&oauth_token.access_token)
            .await?;

        if let Some(user) = self
            .authenticated_user_repository
            .find_by_email(&user_info.email)
            .await?
        {
            warn!(
                user_id = user_id.0,
                email = user.email(),
                "User tried to authenticate with an already used email"
            );
            return Err(AuthenticationError::EmailAlreadyInUse);
        }

        let user = create_user_from_successful_authentication(
            &request,
            user_info.name,
            user_info.email,
            oauth_token,
            class_id,
        );

        self.authenticated_user_repository.save(&user).await?;
        self.user_authentication_request_repository
            .remove_by_csrf_token(request.csrf_token())
            .await?;

        let audit_log_reason = "Assigned student roles by OAuth2 Azure AD authentication";

        self.discord_port
            .remove_user_from_class_roles(user_id, Some(audit_log_reason))
            .await?;
        for role in &self.additional_student_roles {
            self.discord_port
                .remove_user_from_role(user_id, *role, Some(audit_log_reason))
                .await?;
        }

        self.discord_port
            .assign_user_to_class_role(user_id, user.class_id(), Some(audit_log_reason))
            .await?;
        for role in &self.additional_student_roles {
            self.discord_port
                .assign_user_to_role(user_id, *role, Some(audit_log_reason))
                .await?;
        }

        info!(user_id = user_id.0, "User successfully authenticated");

        Ok(self.invite_link.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use domain::authentication::authenticated_user::{
        AuthenticatedUser, AuthenticatedUserSnapshot, MockAuthenticatedUserRepository,
    };
    use domain::authentication::user_authentication_request::MockUserAuthenticationRequestRepository;
    use domain::ports::discord::MockDiscordPort;
    use domain::ports::oauth::{MockOAuthPort, OAuthToken, UserInfoDto};
    use domain_shared::authentication::{AccessToken, RefreshToken, UserGroup};
    use std::sync::Arc;
    use tokio;

    #[instrument(level = "trace", skip_all)]
    fn setup_mocks() -> (
        MockDiscordPort,
        MockOAuthPort,
        MockAuthenticatedUserRepository,
        MockUserAuthenticationRequestRepository,
    ) {
        let discord_port = MockDiscordPort::new();
        let oauth_port = MockOAuthPort::new();
        let authenticated_user_repository = MockAuthenticatedUserRepository::new();
        let user_authentication_request_repository = MockUserAuthenticationRequestRepository::new();

        (
            discord_port,
            oauth_port,
            authenticated_user_repository,
            user_authentication_request_repository,
        )
    }

    #[instrument(level = "trace", skip_all)]
    fn create_service(
        discord_port: MockDiscordPort,
        oauth_port: MockOAuthPort,
        authenticated_user_repository: MockAuthenticatedUserRepository,
        user_authentication_request_repository: MockUserAuthenticationRequestRepository,
    ) -> AuthenticationService {
        let invite_link = InviteLink("http://discord.gg/invite".into());
        let additional_student_roles = vec![RoleId(123), RoleId(456)];
        AuthenticationService::new(
            Arc::new(discord_port),
            Arc::new(oauth_port),
            Arc::new(authenticated_user_repository),
            Arc::new(user_authentication_request_repository),
            invite_link,
            additional_student_roles,
        )
    }

    #[tokio::test]
    async fn create_authentication_link_already_authenticated() {
        let (discord_port, oauth_port, mut authenticated_user_repo, user_auth_request_repo) =
            setup_mocks();
        let user_id = UserId(12345);

        let name = "Test User".to_string();
        let email = "test@example.com".to_string();
        let class_id = "1a".to_string();
        let expires_at = Utc::now() + Duration::hours(1);
        let oauth_token = OAuthToken {
            access_token: AccessToken("access".to_string()),
            refresh_token: RefreshToken("refresh".to_string()),
            expires_at,
        };
        let authenticated_at = Utc::now();
        let existing_user = AuthenticatedUser::from_snapshot(AuthenticatedUserSnapshot {
            user_id,
            name: name.clone(),
            email: email.clone(),
            class_id: class_id.clone(),
            oauth_token: oauth_token.clone(),
            authenticated_at,
        });
        let existing_user_snapshot = existing_user.to_snapshot();

        authenticated_user_repo
            .expect_find_by_user_id()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .returning(move |_| {
                Ok(Some(AuthenticatedUser::from_snapshot(
                    existing_user_snapshot.clone(),
                )))
            });

        let service = create_service(
            discord_port,
            oauth_port,
            authenticated_user_repo,
            user_auth_request_repo,
        );

        let result = service.create_authentication_link(user_id).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AuthenticationError::AlreadyAuthenticated => {}
            other => panic!("Expected AlreadyAuthenticated, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn create_authentication_link_success() {
        let (discord_port, mut oauth_port, mut authenticated_user_repo, mut user_auth_request_repo) =
            setup_mocks();
        let user_id = UserId(98765);
        let expected_link = AuthenticationLink("http://oauth.com/auth?csrf=...&...".to_string());
        let expected_csrf_token = CsrfToken("random_csrf_token".to_string());

        authenticated_user_repo
            .expect_find_by_user_id()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .returning(|_| Ok(None));

        let link_clone = expected_link.0.clone();
        let csrf_clone = expected_csrf_token.clone();
        oauth_port
            .expect_create_authentication_link()
            .times(1)
            .returning(move || Ok((AuthenticationLink(link_clone.clone()), csrf_clone.clone())));

        let csrf_clone_for_save = expected_csrf_token.clone();
        user_auth_request_repo
            .expect_save()
            .withf(move |req| req.user_id() == user_id && req.csrf_token() == &csrf_clone_for_save)
            .times(1)
            .returning(|_| Ok(()));

        let service = create_service(
            discord_port,
            oauth_port,
            authenticated_user_repo,
            user_auth_request_repo,
        );

        let result = service.create_authentication_link(user_id).await;

        assert!(result.is_ok());
        let returned_link = result.unwrap();
        assert_eq!(returned_link.0, expected_link.0);
    }

    #[tokio::test]
    async fn confirm_authentication_request_not_found() {
        let (
            discord_port,
            oauth_port,
            authenticated_user_repo,
            mut user_auth_request_repo, // mut because expect_find_by_csrf_token is called
        ) = setup_mocks();

        let csrf_token = CsrfToken("non_existent_token".to_string());
        let client_callback_token = ClientCallbackToken("callback_code".to_string());

        user_auth_request_repo
            .expect_find_by_csrf_token()
            .with(mockall::predicate::eq(csrf_token.clone()))
            .times(1)
            .returning(|_| Ok(None)); // Request not found

        let service = create_service(
            discord_port,
            oauth_port,
            authenticated_user_repo,
            user_auth_request_repo,
        );

        let result = service
            .confirm_authentication(csrf_token, client_callback_token)
            .await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AuthenticationError::AuthenticationRequestNotFound => {}
            other => panic!("Expected AuthenticationRequestNotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn confirm_authentication_email_already_in_use() {
        let (discord_port, mut oauth_port, mut authenticated_user_repo, mut user_auth_request_repo) =
            setup_mocks();

        let user_id = UserId(12345); // ID from the original request
        let csrf_token = CsrfToken("valid_csrf_token".to_string());
        let client_callback_token = ClientCallbackToken("callback_code".to_string());
        let oauth_access_token = AccessToken("oauth_access".to_string());
        let oauth_refresh_token = RefreshToken("oauth_refresh".to_string());
        let oauth_expires_at = Utc::now() + Duration::hours(1);
        let oauth_token = OAuthToken {
            access_token: oauth_access_token.clone(),
            refresh_token: oauth_refresh_token.clone(),
            expires_at: oauth_expires_at,
        };
        let class_group_id = "1a".to_string();
        let class_group_mail = format!("{}@ssps.cz", class_group_id);
        let user_groups = vec![UserGroup {
            id: "group_id".to_string(),
            name: class_group_id.clone(),
            mail: Some(class_group_mail.clone()),
        }];
        let user_name = "Test User".to_string();
        let user_email = "existing@example.com".to_string();
        let existing_user_id = UserId(99999); // Different user ID

        // Mock finding the original request
        use domain::authentication::user_authentication_request::UserAuthenticationRequest;
        use domain::authentication::user_authentication_request::UserAuthenticationRequestSnapshot;
        let request_time = Utc::now() - Duration::minutes(5);
        let request = UserAuthenticationRequest::from_snapshot(UserAuthenticationRequestSnapshot {
            csrf_token: csrf_token.clone(),
            user_id, // User ID from request
            requested_at: request_time,
        });
        let request_snapshot = request.to_snapshot();
        user_auth_request_repo
            .expect_find_by_csrf_token()
            .with(mockall::predicate::eq(csrf_token.clone()))
            .times(1)
            .returning(move |_| {
                Ok(Some(UserAuthenticationRequest::from_snapshot(
                    request_snapshot.clone(),
                )))
            });

        // Mock OAuth flow
        let oauth_token_clone = oauth_token.clone();
        oauth_port
            .expect_exchange_code_after_callback()
            .with(mockall::predicate::eq(client_callback_token.clone())) // Need to clone ClientCallbackToken too
            .times(1)
            .returning(move |_| Ok(oauth_token_clone.clone()));

        let user_groups_clone = user_groups.clone();
        oauth_port
            .expect_get_user_groups()
            .with(mockall::predicate::eq(oauth_access_token.clone()))
            .times(1)
            .returning(move |_| Ok(user_groups_clone.clone()));

        let user_name_clone = user_name.clone();
        let user_email_clone = user_email.clone();
        oauth_port
            .expect_get_user_info()
            .with(mockall::predicate::eq(oauth_access_token.clone()))
            .times(1)
            .returning(move |_| {
                Ok(UserInfoDto {
                    name: user_name_clone.clone(),
                    email: user_email_clone.clone(),
                })
            });

        // Mock finding an existing user with the same email
        let existing_user_oauth_token = OAuthToken {
            access_token: AccessToken("existing_access".to_string()),
            refresh_token: RefreshToken("existing_refresh".to_string()),
            expires_at: Utc::now() + Duration::hours(1),
        };
        let existing_user = AuthenticatedUser::from_snapshot(AuthenticatedUserSnapshot {
            user_id: existing_user_id,
            name: "Existing User".to_string(),
            email: user_email.clone(), // Same email
            class_id: "2b".to_string(),
            oauth_token: existing_user_oauth_token,
            authenticated_at: Utc::now() - Duration::days(2),
        });
        let existing_user_snapshot_for_find = existing_user.to_snapshot();
        authenticated_user_repo
            .expect_find_by_email()
            .with(mockall::predicate::eq(user_email.clone()))
            .times(1)
            .returning(move |_| {
                Ok(Some(AuthenticatedUser::from_snapshot(
                    existing_user_snapshot_for_find.clone(),
                )))
            });

        // No save or remove should happen

        let service = create_service(
            discord_port,
            oauth_port,
            authenticated_user_repo,
            user_auth_request_repo,
        );

        let result = service
            .confirm_authentication(csrf_token, client_callback_token)
            .await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AuthenticationError::EmailAlreadyInUse => {}
            other => panic!("Expected EmailAlreadyInUse, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn confirm_authentication_success() {
        let (
            mut discord_port, // mut for role changes
            mut oauth_port,
            mut authenticated_user_repo,
            mut user_auth_request_repo,
        ) = setup_mocks();

        let user_id = UserId(55555); // ID from the original request
        let csrf_token = CsrfToken("success_csrf_token".to_string());
        let client_callback_token = ClientCallbackToken("success_callback_code".to_string());
        let oauth_access_token = AccessToken("success_oauth_access".to_string());
        let oauth_refresh_token = RefreshToken("success_oauth_refresh".to_string());
        let oauth_expires_at = Utc::now() + Duration::hours(1);
        let oauth_token = OAuthToken {
            access_token: oauth_access_token.clone(),
            refresh_token: oauth_refresh_token.clone(),
            expires_at: oauth_expires_at,
        };
        let class_group_id = "2a".to_string();
        let class_group_mail = format!("{}@ssps.cz", class_group_id);
        let user_groups = vec![UserGroup {
            id: "group_id_2a".to_string(),
            name: class_group_id.clone(),
            mail: Some(class_group_mail.clone()),
        }];
        let user_name = "Successful User".to_string();
        let user_email = "success@example.com".to_string();
        let expected_invite_link = InviteLink("http://discord.gg/invite".into());
        let additional_role1 = RoleId(123);
        let additional_role2 = RoleId(456);

        let mut seq = mockall::Sequence::new(); // Enforce sequence

        // 1. Find request
        use domain::authentication::user_authentication_request::UserAuthenticationRequest;
        use domain::authentication::user_authentication_request::UserAuthenticationRequestSnapshot;
        let request_time = Utc::now() - Duration::minutes(2);
        let request = UserAuthenticationRequest::from_snapshot(UserAuthenticationRequestSnapshot {
            csrf_token: csrf_token.clone(),
            user_id,
            requested_at: request_time,
        });
        let request_snapshot = request.to_snapshot();
        user_auth_request_repo
            .expect_find_by_csrf_token()
            .with(mockall::predicate::eq(csrf_token.clone()))
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| {
                Ok(Some(UserAuthenticationRequest::from_snapshot(
                    request_snapshot.clone(),
                )))
            });

        // 2. Exchange code
        let oauth_token_clone = oauth_token.clone();
        oauth_port
            .expect_exchange_code_after_callback()
            .with(mockall::predicate::eq(client_callback_token.clone()))
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(oauth_token_clone.clone()));

        // 3. Get groups
        let user_groups_clone = user_groups.clone();
        oauth_port
            .expect_get_user_groups()
            .with(mockall::predicate::eq(oauth_access_token.clone()))
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(user_groups_clone.clone()));

        // 4. Get user info
        let user_name_clone = user_name.clone();
        let user_email_clone = user_email.clone();
        oauth_port
            .expect_get_user_info()
            .with(mockall::predicate::eq(oauth_access_token.clone()))
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| {
                Ok(UserInfoDto {
                    name: user_name_clone.clone(),
                    email: user_email_clone.clone(),
                })
            });

        // 5. Check email not in use
        authenticated_user_repo
            .expect_find_by_email()
            .with(mockall::predicate::eq(user_email.clone()))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(None)); // Email not found

        // 6. Save authenticated user
        let saved_user_token_clone = oauth_token.clone();
        let saved_user_name_clone = user_name.clone();
        let saved_user_email_clone = user_email.clone();
        let saved_user_class_id_clone = class_group_id.clone();
        authenticated_user_repo
            .expect_save()
            .withf(move |user: &AuthenticatedUser| {
                user.user_id() == user_id
                    && user.name() == saved_user_name_clone
                    && user.email() == saved_user_email_clone
                    && user.oauth_token() == &saved_user_token_clone
                    && user.class_id() == saved_user_class_id_clone
                // authenticated_at is Utc::now(), difficult to assert precisely
            })
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));

        // 7. Remove request
        let csrf_clone_for_remove = csrf_token.clone();
        user_auth_request_repo
            .expect_remove_by_csrf_token()
            .with(mockall::predicate::eq(csrf_clone_for_remove))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));

        // 8. Discord remove roles
        discord_port
            .expect_remove_user_from_class_roles()
            .with(
                mockall::predicate::eq(user_id),
                mockall::predicate::always(),
            )
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _| Ok(()));
        discord_port
            .expect_remove_user_from_role()
            .with(
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(additional_role1),
                mockall::predicate::always(),
            )
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _, _| Ok(()));
        discord_port
            .expect_remove_user_from_role()
            .with(
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(additional_role2),
                mockall::predicate::always(),
            )
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _, _| Ok(()));

        // 9. Discord assign roles
        let assigned_class_id_clone = class_group_id.clone();
        discord_port
            .expect_assign_user_to_class_role()
            .withf(move |uid, cid, _| *uid == user_id && cid == assigned_class_id_clone)
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _, _| Ok(()));
        discord_port
            .expect_assign_user_to_role()
            .with(
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(additional_role1),
                mockall::predicate::always(),
            )
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _, _| Ok(()));
        discord_port
            .expect_assign_user_to_role()
            .with(
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(additional_role2),
                mockall::predicate::always(),
            )
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _, _| Ok(()));

        let service = create_service(
            discord_port,
            oauth_port,
            authenticated_user_repo,
            user_auth_request_repo,
        );

        let result = service
            .confirm_authentication(csrf_token, client_callback_token)
            .await;

        assert!(result.is_ok());
        let returned_link = result.unwrap();
        assert_eq!(returned_link.0, expected_invite_link.0);
    }
}
