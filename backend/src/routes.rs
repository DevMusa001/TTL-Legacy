use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, Response, StatusCode},
    middleware::Next,
    Json,
};
use chrono::DateTime;
use serde::Deserialize;
use tracing::instrument;

use crate::{
    audit,
    db::Db,
    error::AppError,
    handlers::{parse_scenario_types, simulate_release_handler, export_vaults_handler},
    models::{ReminderPreferences, SetPreferencesRequest, SimulateReleaseQuery, SimulateReleaseResponse},
};

#[derive(Deserialize)]
pub struct RemindersQuery {
    pub include_deleted: Option<bool>,
}

#[instrument(skip(state), fields(vault_id = %vault_id))]
pub async fn list_vault_reminders(
    State(state): State<Arc<AppState>>,
    Path(vault_id): Path<u64>,
    Query(query): Query<RemindersQuery>,
) -> Result<Json<Vec<ReminderPreferences>>, AppError> {
    let db = &state.db;
    let records = if query.include_deleted.unwrap_or(false) {
        db.all_reminders_including_deleted(vault_id)?
    } else {
        match db.get(vault_id) {
            Ok(p) => vec![p],
            Err(_) => vec![],
        }
    };
    Ok(Json(records))
}

#[instrument(skip(state), fields(vault_id = %vault_id))]
pub async fn delete_preferences(
    State(state): State<Arc<AppState>>,
    Path(vault_id): Path<u64>,
) -> Result<StatusCode, AppError> {
    state.db.soft_delete_reminder(vault_id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(state, headers), fields(vault_id = %vault_id))]
pub async fn set_preferences(
    State(state): State<Arc<AppState>>,
    Path(vault_id): Path<u64>,
    headers: HeaderMap,
    Json(body): Json<SetPreferencesRequest>,
) -> Result<(StatusCode, Json<ReminderPreferences>), AppError> {
    let db = &state.db;
    if body.channels.is_empty() {
        return Err(AppError::InvalidInput("channels must not be empty".into()));
    }
    if body.hours_before_expiry == 0 {
        return Err(AppError::InvalidInput(
            "hours_before_expiry must be > 0".into(),
        ));
    }

    // #825: Idempotency key support
    if let Some(idem_key) = headers.get("idempotency-key").and_then(|v| v.to_str().ok()) {
        if let Some(cached) = db.check_idempotency(idem_key) {
            let cached_prefs: ReminderPreferences =
                serde_json::from_str(&cached.response_body).unwrap();
            return Ok((StatusCode::OK, Json(cached_prefs)));
        }
    }

    let prefs = ReminderPreferences {
        vault_id,
        channels: body.channels,
        hours_before_expiry: body.hours_before_expiry,
        frequency: body.frequency,
        deleted_at: None,
    };
    db.upsert(&prefs)?;

    // Store idempotency record if key was provided
    if let Some(idem_key) = headers.get("idempotency-key").and_then(|v| v.to_str().ok()) {
        let body_json = serde_json::to_string(&prefs).unwrap();
        db.store_idempotency(idem_key, 200, &body_json);
    }

    Ok((StatusCode::OK, Json(prefs)))
}

#[instrument(skip(state), fields(vault_id = %vault_id))]
pub async fn get_preferences(
    State(state): State<Arc<AppState>>,
    Path(vault_id): Path<u64>,
) -> Result<Json<ReminderPreferences>, AppError> {
    let db = &state.db;
    match db.get(vault_id) {
        Ok(prefs) => Ok(Json(prefs)),
        Err(_e) => Err(AppError::NotFound),
    }
}

// ── Unsubscribe endpoint (#828) ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UnsubscribeQuery {
    pub token: String,
}

#[instrument(skip(state))]
pub async fn unsubscribe(
    State(state): State<Arc<AppState>>,
    Query(query): Query<UnsubscribeQuery>,
) -> Result<(StatusCode, String), AppError> {
    let db = &state.db;
    match db.process_unsubscribe(&query.token) {
        Ok(owner) => Ok((
            StatusCode::OK,
            format!("You ({owner}) have been unsubscribed from reminder emails."),
        )),
        Err(_) => Err(AppError::InvalidInput(
            "Invalid or expired unsubscribe token".into(),
        )),
    }
}


// ── Release Simulator endpoint ────────────────────────────────────────────────

/// GET /api/vaults/:vault_id/simulate-release?scenarios=no_check_ins,consistent_check_ins,missed_check_in_dates&missed_count=2
#[instrument(skip(db), fields(vault_id = %vault_id))]
pub async fn simulate_release(
    State(db): State<Arc<Db>>,
    Path(vault_id): Path<String>,
    Query(query): Query<SimulateReleaseQuery>,
) -> Result<Json<SimulateReleaseResponse>, AppError> {
    let scenarios = parse_scenario_types(query.scenarios.as_deref());
    if scenarios.is_empty() {
        return Err(AppError::InvalidInput(
            "No valid scenarios requested. Use: no_check_ins, consistent_check_ins, missed_check_in_dates".into(),
        ));
    }

    let missed_count = query.missed_count.unwrap_or(1);

    let result = simulate_release_handler(
        &db.vault_store,
        &vault_id,
        scenarios,
        missed_count,
    )
    .map_err(|_| AppError::NotFound)?;

    Ok(Json(result))
}


// ── Sponsored Release endpoints (#1122) ──────────────────────────────────────

use crate::fee_sponsorship::{SponsoredReleaseRequest, SponsoredReleaseResponse};
use crate::handlers::{sponsored_release_handler, get_sponsored_release_handler, list_sponsored_releases_handler};

/// POST /api/vaults/:vault_id/sponsored-release
/// Create a sponsored release transaction for a beneficiary.
pub async fn create_sponsored_release(
    State(state): State<Arc<AppState>>,
    Path(vault_id): Path<String>,
    Json(req): Json<SponsoredReleaseRequest>,
) -> Result<(StatusCode, Json<SponsoredReleaseResponse>), AppError> {
    let result = sponsored_release_handler(
        &state.db.vault_store,
        Arc::clone(&state.db),
        &vault_id,
        req,
    )
    .map_err(|e| AppError::InvalidInput(e))?;

    Ok((StatusCode::CREATED, Json(result)))
}

/// GET /api/vaults/:vault_id/sponsored-release
/// List all sponsored releases for a vault.
pub async fn get_sponsored_releases(
    State(state): State<Arc<AppState>>,
    Path(vault_id): Path<String>,
) -> Result<Json<Vec<crate::fee_sponsorship::SponsoredRelease>>, AppError> {
    let result = list_sponsored_releases_handler(Arc::clone(&state.db), &vault_id)
        .map_err(|e| AppError::InvalidInput(e))?;

    Ok(Json(result))
}

// ── Audit Log Export (#1128) ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ExportQuery {
    pub format: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// GET /api/vaults/:vault_id/export?format=csv|json&from=&to=
#[instrument(skip(state, headers), fields(vault_id = %vault_id))]
pub async fn export_vault(
    State(state): State<Arc<AppState>>,
    Path(vault_id): Path<String>,
    Query(query): Query<ExportQuery>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let user_id = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let format = match query.format.as_str() {
        "csv" | "json" => query.format,
        _ => {
            return Err(AppError::InvalidInput(
                "format must be csv or json".into(),
            ))
        }
    };

    let from = query
        .from
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let to = query
        .to
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let result = export_vaults_handler(
        &state.db.vault_store,
        &state.db.event_store,
        &state.db.audit_store,
        &vault_id,
        &format,
        from,
        to,
        if user_id.is_empty() { None } else { Some(user_id) },
    )
    .map_err(|e| AppError::InvalidInput(e))?;

    let content_type = if format == "csv" {
        "text/csv"
    } else {
        "application/json"
    };

    Ok(Response::builder()
        .header("Content-Type", content_type)
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", format!("vault-{}.{}", vault_id, format)))
        .body(Body::from(result))
        .unwrap())
}

// ── Rate Limited Stub Endpoints (#1127) ──────────────────────────────────────

pub async fn stub_checkin() -> Json<serde_json::Value> {
    Json(serde_json::json!({"error": "not_implemented", "message": "checkin endpoint not yet available"}))
}

pub async fn stub_release() -> Json<serde_json::Value> {
    Json(serde_json::json!({"error": "not_implemented", "message": "release endpoint not yet available"}))
}

pub async fn stub_email_token() -> Json<serde_json::Value> {
    Json(serde_json::json!({"error": "not_implemented", "message": "email-token endpoint not yet available"}))
}
