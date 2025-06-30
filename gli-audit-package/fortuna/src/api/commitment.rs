// This module provides an endpoint to retrieve the provider's initial commitment for a given chain.

use crate::api::{ApiBlockChainState, ChainId, RestError};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct GetCommitmentResponse {
    pub commitment: String,
}

#[utoipa::path(
    get,
    path = "/v1/commitment/{chain_id}",
    responses(
        (status = 200, description = "Provider commitment successfully retrieved", body = GetCommitmentResponse),
        (status = 400, description = "Invalid chain id", body = String)
    ),
    params(
        ("chain_id" = String, Path, description = "ID of the blockchain")
    )
)]
pub async fn commitment(
    State(state): State<crate::api::ApiState>,
    Path(params): Path<ChainId>,
) -> Result<Json<GetCommitmentResponse>, RestError> {
    let chain_state = state
        .chains
        .read()
        .await
        .get(&params)
        .ok_or(RestError::InvalidChainId)?
        .clone();

    let chain_state = match chain_state {
        ApiBlockChainState::Initialized(state) => state,
        ApiBlockChainState::Uninitialized => {
            return Err(RestError::Uninitialized);
        }
    };

    // The commitment is the 0'th revealed value in the hash chain.
    let commitment = chain_state.state.reveal(0).map_err(|_| RestError::Uninitialized)?;

    Ok(Json(GetCommitmentResponse {
        commitment: hex::encode(commitment),
    }))
} 