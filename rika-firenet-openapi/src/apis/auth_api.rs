/*
 * Rika Firenet - OpenAPI 3.0
 *
 * Rika Firenet API operations
 *
 * The version of the OpenAPI document: 1.0.0
 *
 * Generated by: https://openapi-generator.tech
 */

use reqwest;

use super::{configuration, Error};
use crate::apis::ResponseContent;

/// struct for passing parameters to the method [`login`]
#[derive(Clone, Debug)]
pub struct LoginParams {
    /// Rika Firenet account email
    pub email: String,
    /// Rika Firenet password
    pub password: String,
}

/// struct for typed errors of method [`login`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LoginError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`logout`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LogoutError {
    UnknownValue(serde_json::Value),
}

pub async fn login(
    configuration: &configuration::Configuration,
    params: LoginParams,
) -> Result<(), Error<LoginError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let email = params.email;
    let password = params.password;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/web/login", local_var_configuration.base_path);
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    let mut local_var_form_params = std::collections::HashMap::new();
    local_var_form_params.insert("email", email.to_string());
    local_var_form_params.insert("password", password.to_string());
    local_var_req_builder = local_var_req_builder.form(&local_var_form_params);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<LoginError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn logout(
    configuration: &configuration::Configuration,
) -> Result<(), Error<LogoutError>> {
    let local_var_configuration = configuration;

    // unbox the parameters

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/web/logout", local_var_configuration.base_path);
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<LogoutError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}