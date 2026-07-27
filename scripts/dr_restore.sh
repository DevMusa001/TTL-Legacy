#!/usr/bin/env bash
# dr_restore.sh — Disaster Recovery Restore Script
#
# Restores the TTL-Legacy backend database from a snapshot stored in an
# S3-compatible bucket and re-deploys the contract from a backed-up WASM.
#
# Usage:
#   ./scripts/dr_restore.sh --backup-prefix <PREFIX> [--dry-run]
#
#   <PREFIX>  The timestamp-prefixed path inside the bucket
#             (e.g. backups/20260727T120000Z)
#
# Required environment variables:
#   DR_S3_BUCKET          S3-compatible bucket name
#   DR_S3_ENDPOINT        S3 endpoint URL
#   STELLAR_NETWORK       Stellar network (testnet | mainnet | standalone)
#   DEPLOYER_IDENTITY     Stellar CLI key name to sign the re-deployment
#   DR_DB_PATH            Destination path for the restored database
#
# Optional environment variables:
#   DR_S3_PREFIX          Key prefix inside the bucket  (default: backups)
#   AWS_ACCESS_KEY_ID     S3 credentials
#   AWS_SECRET_ACCESS_KEY S3 credentials
#   AWS_REGION            AWS region                    (default: us-east-1)

set -euo pipefail

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

log()  { echo "[dr_restore] $*"; }
warn() { echo "[dr_restore] WARNING: $*" >&2; }
die()  { echo "[dr_restore] ERROR: $*" >&2; exit 1; }

DRY_RUN=false
BACKUP_PREFIX=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --backup-prefix)
            BACKUP_PREFIX="$2"
            shift 2
            ;;
        *)
            die "Unknown argument: $1"
            ;;
    esac
done

if $DRY_RUN; then
    log "Running in DRY-RUN mode — no changes will be made."
fi

# ---------------------------------------------------------------------------
# Validate required variables
# ---------------------------------------------------------------------------

: "${DR_S3_BUCKET:?DR_S3_BUCKET is required}"
: "${DR_S3_ENDPOINT:?DR_S3_ENDPOINT is required}"
: "${STELLAR_NETWORK:?STELLAR_NETWORK is required}"
: "${DEPLOYER_IDENTITY:?DEPLOYER_IDENTITY is required}"
: "${DR_DB_PATH:?DR_DB_PATH is required}"

AWS_REGION="${AWS_REGION:-us-east-1}"

if [[ -z "${BACKUP_PREFIX}" ]]; then
    die "--backup-prefix is required. Run dr_backup.sh first and note the printed prefix."
fi

log "Starting disaster-recovery restore"
log "  Network         : ${STELLAR_NETWORK}"
log "  Backup prefix   : s3://${DR_S3_BUCKET}/${BACKUP_PREFIX}/"
log "  DB destination  : ${DR_DB_PATH}"

run() {
    if $DRY_RUN; then
        log "[DRY-RUN] Would run: $*"
    else
        "$@"
    fi
}

# ---------------------------------------------------------------------------
# 1. Download backup manifest
# ---------------------------------------------------------------------------

MANIFEST_FILE=$(mktemp /tmp/manifest_XXXXXX.json)
trap 'rm -f "${MANIFEST_FILE}" "${WASM_FILE:-}" "${DB_FILE:-}"' EXIT

log "Downloading manifest from s3://${DR_S3_BUCKET}/${BACKUP_PREFIX}/manifest.json ..."
if $DRY_RUN; then
    log "[DRY-RUN] Would download manifest"
    MANIFEST_NETWORK="${STELLAR_NETWORK}"
else
    aws s3 cp \
        "s3://${DR_S3_BUCKET}/${BACKUP_PREFIX}/manifest.json" \
        "${MANIFEST_FILE}" \
        --endpoint-url "${DR_S3_ENDPOINT}" \
        --region "${AWS_REGION}" \
        || die "Failed to download manifest"

    MANIFEST_NETWORK=$(python3 -c "import json,sys; d=json.load(open('${MANIFEST_FILE}')); print(d['network'])" 2>/dev/null || echo "")
    log "Manifest network: ${MANIFEST_NETWORK}"

    if [[ -n "${MANIFEST_NETWORK}" && "${MANIFEST_NETWORK}" != "${STELLAR_NETWORK}" ]]; then
        warn "Manifest network (${MANIFEST_NETWORK}) differs from STELLAR_NETWORK (${STELLAR_NETWORK})."
        warn "Continuing — make sure you intended to restore across networks."
    fi
fi

# ---------------------------------------------------------------------------
# 2. Restore backend database
# ---------------------------------------------------------------------------

DB_FILE=$(mktemp /tmp/restore_db_XXXXXX.sqlite3)
log "Downloading database snapshot..."
if $DRY_RUN; then
    log "[DRY-RUN] Would download s3://${DR_S3_BUCKET}/${BACKUP_PREFIX}/backend_db.sqlite3"
    log "[DRY-RUN] Would restore to ${DR_DB_PATH}"
else
    aws s3 cp \
        "s3://${DR_S3_BUCKET}/${BACKUP_PREFIX}/backend_db.sqlite3" \
        "${DB_FILE}" \
        --endpoint-url "${DR_S3_ENDPOINT}" \
        --region "${AWS_REGION}" \
        || die "Failed to download database snapshot"

    log "Database snapshot downloaded ($(wc -c < "${DB_FILE}") bytes)"

    # Back up existing database before overwriting
    if [[ -f "${DR_DB_PATH}" ]]; then
        EXISTING_BACKUP="${DR_DB_PATH}.pre_restore_$(date -u +"%Y%m%dT%H%M%SZ")"
        log "Backing up existing database to ${EXISTING_BACKUP}"
        cp "${DR_DB_PATH}" "${EXISTING_BACKUP}"
    fi

    # Restore
    cp "${DB_FILE}" "${DR_DB_PATH}"
    log "Database restored to ${DR_DB_PATH}"
fi

# ---------------------------------------------------------------------------
# 3. Re-deploy contract from backed-up WASM
# ---------------------------------------------------------------------------

WASM_FILE=$(mktemp /tmp/restore_wasm_XXXXXX.wasm)
log "Downloading WASM from s3://${DR_S3_BUCKET}/${BACKUP_PREFIX}/ttl_vault.wasm ..."
if $DRY_RUN; then
    log "[DRY-RUN] Would download s3://${DR_S3_BUCKET}/${BACKUP_PREFIX}/ttl_vault.wasm"
    log "[DRY-RUN] Would run: stellar contract deploy --wasm <wasm> --source ${DEPLOYER_IDENTITY} --network ${STELLAR_NETWORK}"
else
    aws s3 cp \
        "s3://${DR_S3_BUCKET}/${BACKUP_PREFIX}/ttl_vault.wasm" \
        "${WASM_FILE}" \
        --endpoint-url "${DR_S3_ENDPOINT}" \
        --region "${AWS_REGION}" \
        || die "Failed to download WASM artefact"

    log "WASM downloaded ($(wc -c < "${WASM_FILE}") bytes). Deploying contract..."

    NEW_CONTRACT_ID=$(stellar contract deploy \
        --wasm "${WASM_FILE}" \
        --source "${DEPLOYER_IDENTITY}" \
        --network "${STELLAR_NETWORK}") \
        || die "Contract re-deployment failed"

    log "Contract re-deployed: ${NEW_CONTRACT_ID}"
    log "Update CONTRACT_TTL_VAULT=${NEW_CONTRACT_ID} in your environment / environments.toml"
fi

# ---------------------------------------------------------------------------
# 4. Verify restored database is readable
# ---------------------------------------------------------------------------

if ! $DRY_RUN; then
    if command -v sqlite3 &>/dev/null; then
        TABLE_COUNT=$(sqlite3 "${DR_DB_PATH}" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';" 2>/dev/null || echo "?")
        log "Post-restore DB check: ${TABLE_COUNT} table(s) found"
    else
        warn "sqlite3 not in PATH — skipping DB integrity check"
    fi
fi

log "Disaster-recovery restore complete."
if $DRY_RUN; then
    log "This was a dry run. Re-run without --dry-run to apply changes."
fi
