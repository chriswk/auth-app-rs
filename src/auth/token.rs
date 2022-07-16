use crate::errors::AuthAppError;
use crate::AppConfig;
use itertools::Itertools;
use rusty_paseto::prelude::*;

pub fn create_token(
    config: AppConfig,
    email: String,
    client_ids: Vec<String>,
) -> Result<String, AuthAppError> {
    let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(config.secret.as_bytes()));
    PasetoBuilder::<V4, Local>::default()
        .set_claim(
            CustomClaim::try_from(("client_ids", client_ids.iter().join(",")))
                .expect("Couldn't set client_id claim"),
        )
        .set_claim(CustomClaim::try_from(("email", email)).expect("Couldn't set email claim"))
        .build(&key)
        .map_err(|_| AuthAppError::InvalidToken)
}

pub struct AuthenticatedTokenUser {
    pub email: String,
    pub client_ids: Vec<String>,
}

pub fn validate_token(
    config: AppConfig,
    token: String,
) -> Result<AuthenticatedTokenUser, AuthAppError> {
    let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(config.secret.as_bytes()));
    let token_result = PasetoParser::<V4, Local>::default().parse(&token, &key);
    match token_result {
        Ok(token_value) => {
            let email = token_value["email"].as_str();
            let client_ids = token_value["client_ids"]
                .as_str()
                .map(|ids| Vec::from_iter(ids.split(',').into_iter().map(|c| c.to_string())));

            email
                .zip(client_ids)
                .map(|(e, c)| {
                    Ok(AuthenticatedTokenUser {
                        email: e.to_string(),
                        client_ids: c,
                    })
                })
                .unwrap_or(Err(AuthAppError::InvalidToken))
        }
        Err(_) => Err(AuthAppError::InvalidToken),
    }
}

#[cfg(test)]
#[test]
fn should_be_able_to_create_and_parse_token() {
    let config = AppConfig {
        secret: "abc123abc123abc123abc123abc12323".to_string(),
        ..Default::default()
    };
    let token = create_token(
        config.clone(),
        "test@test.com".to_string(),
        vec!["admin".to_string(), "demo".to_string(), "test".to_string()],
    );
    assert!(token.is_ok());
    let authenticated_user = validate_token(config, token.unwrap());
    assert!(authenticated_user.is_ok());
    let u = authenticated_user.unwrap();
    assert_eq!(u.email, "test@test.com".to_string());
    assert_eq!(u.client_ids.len(), 3);
}
