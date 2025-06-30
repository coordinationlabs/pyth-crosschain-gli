// This module provides a mock revelation endpoint for testing purposes.
// It bypasses the on-chain request checks and directly reveals a value from the hash chain.

use crate::api::{ApiBlockChainState, ChainId, GetRandomValueResponse, RestError, Blob, BinaryEncoding};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(
    get,
    path = "/v1/chains/{chain_id}/mock_revelation/{sequence}",
    responses(
        (status = 200, description = "Mock random value successfully retrieved", body = GetRandomValueResponse),
        (status = 400, description = "Invalid chain or sequence number", body = String)
    ),
    params(
        ("chain_id" = String, Path, description = "ID of the blockchain"),
        ("sequence" = u64, Path, description = "Sequence number to reveal"),
        ("encoding" = Option<BinaryEncoding>, Query, description = "Encoding for the revealed value (hex, base64, array)")
    )
)]
pub async fn mock_revelation(
    State(state): State<crate::api::ApiState>,
    Path((chain_id, sequence)): Path<(ChainId, u64)>,
    Query(params): Query<MockRevelationQueryParams>,
) -> Result<Json<GetRandomValueResponse>, RestError> {

    let chain_state = state
        .chains
        .read()
        .await
        .get(&chain_id)
        .ok_or(RestError::InvalidChainId)?
        .clone();

    let chain_state = match chain_state {
        ApiBlockChainState::Initialized(state) => state,
        ApiBlockChainState::Uninitialized => {
            return Err(RestError::Uninitialized);
        }
    };

    let value = &chain_state.state.reveal(sequence).map_err(|e| {
        tracing::error!(
            chain_id = chain_id,
            sequence = sequence,
            "Mock reveal failed {}",
            e
        );
        RestError::Unknown
    })?;
    
    let encoded_value = Blob::new(params.encoding.unwrap_or(BinaryEncoding::Hex), *value);

    Ok(Json(GetRandomValueResponse {
        value: encoded_value,
    }))
}

#[derive(Debug, serde::Deserialize, IntoParams)]
pub struct MockRevelationQueryParams {
    encoding: Option<BinaryEncoding>,
} 