/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2022
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */

use crate::{
    db::{ApiAuthed, DB},
    users::{maybe_refresh_folders, require_owner_of_path},
    utils::check_scopes,
    webhook_util::{WebhookMessage, WebhookShared},
};

use axum::{
    extract::{Extension, Path, Query},
    routing::{delete, get, post},
    Json, Router,
};
use hyper::StatusCode;
use serde_json::Value;

use windmill_audit::audit_oss::{audit_log, AuditAuthorable};
use windmill_audit::ActionKind;
use windmill_common::{
    db::UserDB, error::{Error, JsonResult, Result}, utils::{not_found_if_none, paginate, Pagination, StripPath}, variables::{
        build_crypt, get_reserved_variables, ContextualVariable, CreateVariable, ListableVariable,
    },
    worker::CLOUD_HOSTED,
};

use lazy_static::lazy_static;
use serde::Deserialize;
use sqlx::{Postgres, Transaction};
use windmill_common::variables::{decrypt, encrypt};
use windmill_git_sync::{handle_deployment_metadata, DeployedObject};

lazy_static! {
    pub static ref SECRET_SALT: Option<String> = std::env::var("SECRET_SALT").ok();
}

pub fn workspaced_service() -> Router {
    Router::new()
        .route("/list", get(list_variables))
        .route("/list_contextual", get(list_contextual_variables))
        .route("/get/*path", get(get_variable))
        .route("/get_value/*path", get(get_value))
        .route("/exists/*path", get(exists_variable))
        .route("/update/*path", post(update_variable))
        .route("/delete/*path", delete(delete_variable))
        .route("/create", post(create_variable))
        .route("/encrypt", post(encrypt_value))
}

async fn list_contextual_variables(
    Path(w_id): Path<String>,
    ApiAuthed { username, email, .. }: ApiAuthed,
    Extension(db): Extension<DB>,
) -> JsonResult<Vec<ContextualVariable>> {
    Ok(Json(
        get_reserved_variables(
            &db.into(),
            &w_id,
            "q1A0qcPuO00yxioll7iph76N9CJDqn",
            &email,
            &username,
            "017e0ad5-f499-73b6-5488-92a61c5196dd",
            format!("u/{username}").as_str(),
            Some("u/user/script_path".to_string()),
            Some("017e0ad5-f499-73b6-5488-92a61c5196dd".to_string()),
            Some("u/user/encapsulating_flow_path".to_string()),
            Some("u/user/triggering_flow_path".to_string()),
            Some("c".to_string()),
            Some("017e0ad5-f499-73b6-5488-92a61c5196dd".to_string()),
            Some(chrono::offset::Utc::now()),
        )
        .await
        .to_vec(),
    ))
}

#[derive(Deserialize)]
struct ListVariableQuery {
    path_start: Option<String>,
}

async fn list_variables(
    authed: ApiAuthed,
    Extension(user_db): Extension<UserDB>,
    Path(w_id): Path<String>,
    Query(lq): Query<ListVariableQuery>,
    Query(pagination): Query<Pagination>,
) -> JsonResult<Vec<ListableVariable>> {
    let (per_page, offset) = paginate(pagination);

    let mut tx = user_db.begin(&authed).await?;

    let rows = sqlx::query_as::<_, ListableVariable>(
        "SELECT variable.workspace_id, variable.path, CASE WHEN is_secret IS TRUE THEN null ELSE variable.value::text END as value, 
         is_secret, variable.description, variable.extra_perms, account, is_oauth, (now() > account.expires_at) as is_expired,
         account.refresh_error,
         resource.path IS NOT NULL as is_linked,
         account.refresh_token != '' as is_refreshed,
         variable.expires_at
         from variable
         LEFT JOIN account ON variable.account = account.id AND account.workspace_id = $1
         LEFT JOIN resource ON resource.path = variable.path AND resource.workspace_id = $1
         WHERE variable.workspace_id = $1 AND variable.path NOT LIKE 'u/' || $2 || '/secret_arg/%' 
            AND variable.path LIKE $3 || '%'
         ORDER BY path
         LIMIT $4 OFFSET $5
",
    )
    .bind(&w_id)
    .bind(&authed.username)
    .bind(&lq.path_start.unwrap_or_default())
    .bind(per_page as i32)
    .bind(offset as i32)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
struct GetVariableQuery {
    decrypt_secret: Option<bool>,
    include_encrypted: Option<bool>,
}

async fn get_variable(
    authed: ApiAuthed,
    Extension(user_db): Extension<UserDB>,
    Extension(db): Extension<DB>,
    Query(q): Query<GetVariableQuery>,
    Path((w_id, path)): Path<(String, StripPath)>,
) -> JsonResult<ListableVariable> {
    let path = path.to_path();
    check_scopes(&authed, || format!("variables:read:{}", path))?;
    let mut tx = user_db.begin(&authed).await?;

    let variable_o = sqlx::query_as::<_, ListableVariable>(
        "SELECT variable.*, (now() > account.expires_at) as is_expired, account.refresh_error,
        resource.path IS NOT NULL as is_linked,
        account.refresh_token != '' as is_refreshed
        from variable
        LEFT JOIN account ON variable.account = account.id
        LEFT JOIN resource ON resource.path = variable.path AND resource.workspace_id = $2
        WHERE variable.path = $1 AND variable.workspace_id = $2
        LIMIT 1",
    )
    .bind(&path)
    .bind(&w_id)
    .fetch_optional(&mut *tx)
    .await?;

    let variable = if let Some(variable) = variable_o {
        variable
    } else {
        explain_variable_perm_error(&path, &w_id, &db).await?;
        unreachable!()
    };

    let decrypt_secret = q.decrypt_secret.unwrap_or(true);

    let r = if variable.is_secret {
        if decrypt_secret {
            audit_log(
                &mut *tx,
                &authed,
                "variables.decrypt_secret",
                ActionKind::Execute,
                &w_id,
                Some(&variable.path),
                None,
            )
            .await?;
        }

        let value = variable.value.unwrap_or_else(|| "".to_string());
        ListableVariable {
            value: if variable.is_expired.unwrap_or(false) && variable.account.is_some() {
                #[cfg(feature = "oauth2")]
                {
                    Some(
                        crate::oauth2_oss::_refresh_token(
                            tx,
                            &variable.path,
                            &w_id,
                            variable.account.unwrap(),
                            &db,
                        )
                        .await?,
                    )
                }
                #[cfg(not(feature = "oauth2"))]
                return Err(Error::internal_err("Require oauth2 feature".to_string()));
            } else if !value.is_empty() && decrypt_secret {
                let _ = tx.commit().await;
                let mc = build_crypt(&db, &w_id).await?;
                Some(decrypt(&mc, value)?)
            } else if q.include_encrypted.unwrap_or(false) {
                Some(value)
            } else {
                None
            },
            ..variable
        }
    } else {
        variable
    };

    Ok(Json(r))
}

async fn get_value(
    authed: ApiAuthed,
    Extension(user_db): Extension<UserDB>,
    Extension(db): Extension<DB>,

    Path((w_id, path)): Path<(String, StripPath)>,
) -> JsonResult<String> {
    let path = path.to_path();
    check_scopes(&authed, || format!("variables:read:{}", path))?;
    let tx = user_db.begin(&authed).await?;
    return get_value_internal(tx, &db, &w_id, &path, &authed)
        .await
        .map(Json);
}

async fn explain_variable_perm_error(
    path: &str,
    w_id: &str,
    db: &sqlx::Pool<Postgres>,
) -> windmill_common::error::Result<()> {
    let extra_perms = sqlx::query_scalar!(
        "SELECT extra_perms from variable WHERE path = $1 AND workspace_id = $2",
        path,
        w_id
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Variable {} not found", path)))?;
    if path.starts_with("f/") {
        let folder = path.split("/").nth(1).ok_or_else(|| {
            Error::BadRequest(format!(
                "path {} should have at least 2 components separated by /",
                path
            ))
        })?;
        let folder_extra_perms = sqlx::query_scalar!(
            "SELECT extra_perms from folder WHERE name = $1 AND workspace_id = $2",
            folder,
            w_id
        )
        .fetch_optional(db)
        .await?;
        return Err(Error::NotAuthorized(format!(
            "Variable exists but you don't have access to it:\nvariable perms: {}\nfolder perms: {}",
            serde_json::to_string_pretty(&extra_perms).unwrap_or_default(), serde_json::to_string_pretty(&folder_extra_perms).unwrap_or_default()
        )));
    } else {
        return Err(Error::NotAuthorized(format!(
            "Variable exists but you don't have access to it:\nvariable perms: {}",
            serde_json::to_string_pretty(&extra_perms).unwrap_or_default()
        )));
    }
}

async fn exists_variable(
    Extension(db): Extension<DB>,
    Path((w_id, path)): Path<(String, StripPath)>,
) -> JsonResult<bool> {
    let path = path.to_path();

    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM variable WHERE path = $1 AND workspace_id = $2)",
        path,
        w_id
    )
    .fetch_one(&db)
    .await?
    .unwrap_or(false);

    Ok(Json(exists))
}

async fn check_path_conflict(db: &DB, w_id: &str, path: &str) -> Result<()> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM variable WHERE path = $1 AND workspace_id = $2)",
        path,
        w_id
    )
    .fetch_one(db)
    .await?
    .unwrap_or(false);
    if exists {
        return Err(Error::BadRequest(format!(
            "Variable {} already exists",
            path
        )));
    }
    return Ok(());
}

async fn create_variable(
    authed: ApiAuthed,
    Extension(db): Extension<DB>,
    Extension(user_db): Extension<UserDB>,
    Extension(webhook): Extension<WebhookShared>,
    Path(w_id): Path<String>,
    Query(AlreadyEncrypted { already_encrypted }): Query<AlreadyEncrypted>,
    Json(variable): Json<CreateVariable>,
) -> Result<(StatusCode, String)> {
    check_scopes(&authed, || format!("variables:write:{}", variable.path))?;
    if *CLOUD_HOSTED {
        let nb_variables = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM variable WHERE workspace_id = $1",
            &w_id
        )
        .fetch_one(&db)
        .await?;
        if nb_variables.unwrap_or(0) >= 10000 {
            return Err(Error::BadRequest(
                    "You have reached the maximum number of variables (10000) on cloud. Contact support@windmill.dev to increase the limit"
                        .to_string(),
                ));
        }
    }
    let authed = maybe_refresh_folders(&variable.path, &w_id, authed, &db).await;

    check_path_conflict(&db, &w_id, &variable.path).await?;
    let value = if variable.is_secret && !already_encrypted.unwrap_or(false) {
        let mc = build_crypt(&db, &w_id).await?;
        encrypt(&mc, &variable.value)
    } else {
        variable.value
    };

    let mut tx = user_db.begin(&authed).await?;

    sqlx::query!(
        "INSERT INTO variable
            (workspace_id, path, value, is_secret, description, account, is_oauth, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &w_id,
        variable.path,
        value,
        variable.is_secret,
        variable.description,
        variable.account,
        variable.is_oauth.unwrap_or(false),
        variable.expires_at
    )
    .execute(&mut *tx)
    .await?;

    audit_log(
        &mut *tx,
        &authed,
        "variables.create",
        ActionKind::Create,
        &w_id,
        Some(&variable.path),
        None,
    )
    .await?;

    tx.commit().await?;

    handle_deployment_metadata(
        &authed.email,
        &authed.username,
        &db,
        &w_id,
        DeployedObject::Variable { path: variable.path.clone(), parent_path: None },
        Some(format!("Variable '{}' created", variable.path.clone())),
        true,
    )
    .await?;

    webhook.send_message(
        w_id.clone(),
        WebhookMessage::CreateVariable { workspace: w_id, path: variable.path.clone() },
    );

    Ok((
        StatusCode::CREATED,
        format!("variable {} created", variable.path),
    ))
}

async fn encrypt_value(
    Extension(db): Extension<DB>,
    Path(w_id): Path<String>,
    Json(variable): Json<String>,
) -> Result<String> {
    let mc = build_crypt(&db, &w_id).await?;
    let value = encrypt(&mc, &variable);

    Ok(value)
}

async fn delete_variable(
    authed: ApiAuthed,
    Extension(db): Extension<DB>,
    Extension(user_db): Extension<UserDB>,
    Extension(webhook): Extension<WebhookShared>,
    Path((w_id, path)): Path<(String, StripPath)>,
) -> Result<String> {
    let path = path.to_path();
    check_scopes(&authed, || format!("variables:write:{}", path))?;
    let mut tx = user_db.begin(&authed).await?;

    sqlx::query!(
        "DELETE FROM variable WHERE path = $1 AND workspace_id = $2",
        path,
        w_id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "DELETE FROM resource WHERE path = $1 AND workspace_id = $2",
        path,
        w_id
    )
    .execute(&mut *tx)
    .await?;
    audit_log(
        &mut *tx,
        &authed,
        "variables.delete",
        ActionKind::Delete,
        &w_id,
        Some(path),
        None,
    )
    .await?;

    tx.commit().await?;

    handle_deployment_metadata(
        &authed.email,
        &authed.username,
        &db,
        &w_id,
        DeployedObject::Variable { path: path.to_string(), parent_path: Some(path.to_string()) },
        Some(format!("Variable '{}' deleted", path)),
        true,
    )
    .await?;

    webhook.send_message(
        w_id.clone(),
        WebhookMessage::DeleteVariable { workspace: w_id, path: path.to_owned() },
    );

    Ok(format!("variable {} deleted", path))
}

#[derive(Deserialize)]
struct EditVariable {
    path: Option<String>,
    value: Option<String>,
    is_secret: Option<bool>,
    description: Option<String>,
    account: Option<i32>,
}

#[derive(Deserialize)]
struct AlreadyEncrypted {
    already_encrypted: Option<bool>,
}

async fn update_variable(
    authed: ApiAuthed,
    Extension(db): Extension<DB>,
    Extension(user_db): Extension<UserDB>,
    Extension(webhook): Extension<WebhookShared>,
    Path((w_id, path)): Path<(String, StripPath)>,
    Query(AlreadyEncrypted { already_encrypted }): Query<AlreadyEncrypted>,
    Json(ns): Json<EditVariable>,
) -> Result<String> {
    use sql_builder::prelude::*;

    let path = path.to_path();
    check_scopes(&authed, || format!("variables:write:{}", path))?;
    let authed = maybe_refresh_folders(&path, &w_id, authed, &db).await;

    let mut sqlb = SqlBuilder::update_table("variable");
    sqlb.and_where_eq("path", "?".bind(&path));
    sqlb.and_where_eq("workspace_id", "?".bind(&w_id));

    if let Some(npath) = &ns.path {
        sqlb.set_str("path", npath);
    }
    let ns_value_is_none = ns.value.is_none();
    if let Some(nvalue) = ns.value {
        let is_secret = if ns.is_secret.is_some() {
            ns.is_secret.unwrap()
        } else {
            sqlx::query_scalar!(
                "SELECT is_secret from variable WHERE path = $1 AND workspace_id = $2",
                &path,
                &w_id
            )
            .fetch_optional(&db)
            .await?
            .unwrap_or(false)
        };

        let value = if is_secret && !already_encrypted.unwrap_or(false) {
            let mc = build_crypt(&db, &w_id).await?;
            encrypt(&mc, &nvalue)
        } else {
            nvalue
        };
        sqlb.set_str("value", &value);
    }

    if let Some(desc) = ns.description {
        sqlb.set_str("description", &desc);
    }

    if let Some(account_id) = ns.account {
        sqlb.set_str("account", account_id);
    }

    if let Some(nbool) = ns.is_secret {
        let old_secret = sqlx::query_scalar!(
            "SELECT is_secret from variable WHERE path = $1 AND workspace_id = $2",
            &path,
            &w_id
        )
        .fetch_optional(&db)
        .await?
        .unwrap_or(false);
        if old_secret != nbool && ns_value_is_none {
            return Err(Error::BadRequest(
                "cannot change is_secret without updating value too".to_string(),
            ));
        }
        sqlb.set_str("is_secret", nbool);
    }
    sqlb.returning("path");

    // Get old account_id if we're updating the account field
    let old_account_id = if ns.account.is_some() {
        sqlx::query_scalar!(
            "SELECT account FROM variable WHERE path = $1 AND workspace_id = $2",
            &path,
            &w_id
        )
        .fetch_optional(&db)
        .await?
        .flatten()
    } else {
        None
    };

    let mut tx: Transaction<'_, Postgres> = user_db.begin(&authed).await?;

    if let Some(npath) = ns.path {
        if npath != path {
            check_path_conflict(&db, &w_id, &npath).await?;
            require_owner_of_path(&authed, path)?;

            let mut v = sqlx::query_scalar!(
                "SELECT value FROM resource  WHERE path = $1 AND workspace_id = $2",
                path,
                w_id
            )
            .fetch_optional(&mut *tx)
            .await?
            .flatten();

            if let Some(old_v) = v {
                v = Some(replace_path(
                    old_v,
                    &format!("$var:{path}"),
                    &format!("$var:{npath}"),
                ))
            }

            sqlx::query!(
                "UPDATE resource SET path = $1, value = $2, edited_at = now() WHERE path = $3 AND workspace_id = $4",
                npath,
                v,
                path,
                w_id
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    let sql = sqlb.sql().map_err(|e| Error::internal_err(e.to_string()))?;

    let npath_o: Option<String> = sqlx::query_scalar(&sql).fetch_optional(&mut *tx).await?;

    let npath = not_found_if_none(npath_o, "Variable", path)?;

    audit_log(
        &mut *tx,
        &authed,
        "variables.update",
        ActionKind::Update,
        &w_id,
        Some(path),
        None,
    )
    .await?;

    // Clean up old account if it's no longer referenced and different from new account
    if let Some(old_acc_id) = old_account_id {
        if ns.account.is_some() && ns.account != Some(old_acc_id) {
            // Check if old account is still referenced by other variables or resources
            let account_still_used = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM variable WHERE account = $1 AND workspace_id = $2)",
                old_acc_id,
                &w_id
            )
            .fetch_one(&mut *tx)
            .await?
            .unwrap_or(true);

            if !account_still_used {
                // Delete the orphaned account
                sqlx::query!(
                    "DELETE FROM account WHERE id = $1 AND workspace_id = $2",
                    old_acc_id,
                    &w_id
                )
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await?;

    handle_deployment_metadata(
        &authed.email,
        &authed.username,
        &db,
        &w_id,
        DeployedObject::Variable { path: npath.clone(), parent_path: Some(path.to_string()) },
        None,
        true,
    )
    .await?;

    webhook.send_message(
        w_id.clone(),
        WebhookMessage::UpdateVariable {
            workspace: w_id,
            old_path: path.to_owned(),
            new_path: npath.clone(),
        },
    );

    Ok(format!("variable {} updated (npath: {:?})", path, npath))
}

fn replace_path(v: serde_json::Value, path: &str, npath: &str) -> Value {
    match v {
        Value::Object(v) => Value::Object(
            v.into_iter()
                .map(|(k, v)| (k, replace_path(v, path, npath)))
                .collect(),
        ),
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|v| replace_path(v, path, npath))
                .collect(),
        ),
        Value::String(s) if s == path => Value::String(npath.to_owned()),
        _ => v,
    }
}

pub async fn get_value_internal<'c>(
    mut tx: Transaction<'c, Postgres>,
    db: &DB,
    w_id: &str,
    path: &str,
    audit_author: &impl AuditAuthorable,
) -> Result<String> {
    let variable_o = sqlx::query!(
        "SELECT value, account, (now() > account.expires_at) as is_expired, is_secret, path from variable
        LEFT JOIN account ON variable.account = account.id WHERE variable.path = $1 AND variable.workspace_id = $2", path, w_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let variable = if let Some(variable) = variable_o {
        variable
    } else {
        explain_variable_perm_error(path, w_id, db).await?;
        unreachable!()
    };

    let r = if variable.is_secret {
        audit_log(
            &mut *tx,
            audit_author,
            "variables.decrypt_secret",
            ActionKind::Execute,
            &w_id,
            Some(&variable.path),
            None,
        )
        .await?;
        let value = variable.value;
        if variable.is_expired.unwrap_or(false) && variable.account.is_some() {
            #[cfg(feature = "oauth2")]
            {
                crate::oauth2_oss::_refresh_token(
                    tx,
                    &variable.path,
                    &w_id,
                    variable.account.unwrap(),
                    db,
                )
                .await?
            }
            #[cfg(not(feature = "oauth2"))]
            return Err(Error::internal_err("Require oauth2 feature".to_string()));
        } else if !value.is_empty() {
            tx.commit().await?;
            let mc = build_crypt(&db, &w_id).await?;
            decrypt(&mc, value)?
        } else {
            "".to_string()
        }
    } else {
        variable.value
    };

    Ok(r)
}

pub async fn get_variable_or_self(path: String, db: &DB, w_id: &str) -> Result<String> {
    if !path.starts_with("$var:") {
        return Ok(path);
    }
    let path = path.strip_prefix("$var:").unwrap().to_string();

    let record = sqlx::query!(
        "SELECT value, is_secret 
         FROM variable 
         WHERE path = $1 AND workspace_id = $2",
        &path,
        &w_id
    )
    .fetch_optional(db)
    .await?;

    if let Some(record) = record {
        let mut value = record.value;
        if record.is_secret {
            let mc = build_crypt(db, w_id).await?;
            value = decrypt(&mc, value)?;
        }

        Ok(value)
    } else {
        Err(Error::NotFound(format!(
            "Variable not found when resolving `$var:{}`",
            path
        )))
    }
}
