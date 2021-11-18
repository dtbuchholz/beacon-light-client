pub use reqwest::{Client, Error as HttpError, header::ACCEPT};
pub use serde::de::DeserializeOwned;
pub use serde::{Deserialize, Serialize};
pub use serde_json::{self, Error as JsonError, Value, Map};
pub use thiserror::Error;
pub use crate::light_client_types::{Attestation, BlockHeaderData, BlockId, CommitteeData, Epoch, EthSpec, ForkVersionedResponse, GenericResponse, Hash256, LightClientUpdate, RootData, SyncCommitteeByValidatorIndices, SignedBeaconBlock, BeaconState, MainnetEthSpec, Slot};
pub use std::net::SocketAddr;
        
const API_PREFIX: &str = "eth";
const ACCEPT_HEADER: &'static str = "Accept";
const ACCEPT_HEADER_VALUE: &'static str = "application/json";
const ACCEPT_HEADER_VALUE_SSZ: &'static str = "application/ssz";

#[derive(Error, Debug)]
pub enum ApiClientError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("http error: {0}")]
    HttpClientError(#[from] HttpError),
    #[error("json error: {0}")]
    SerdeError(#[from] JsonError),
}

pub type ApiResult<T> = Result<T, ApiClientError>;

#[derive(Clone, Debug)]
pub struct BeaconApiClient {
    pub http_client: Client,
    pub base_url: String
}

pub async fn get_call<T: Serialize + DeserializeOwned>(client: &Client, endpoint: &str) -> ApiResult<T> { 
    let response = client.get(endpoint).header(ACCEPT_HEADER, ACCEPT_HEADER_VALUE).send().await?;
    let result = response.json().await?;

    Ok(result)
    // let body = request.bytes().await?;
    // let result = serde_json::from_slice::<GenericResponse<T>>(&body).map(|resp| resp.data);
    // match result {
    //     Ok(result) => Ok(result),
    //     Err(err) => Err(err.into())
    // }
}

// pub async fn get_call_ssz<T: Serialize + DeserializeOwned>(client: &Client, endpoint: &str) -> ApiResult<T> { 
//     let request = client.get(endpoint).header(ACCEPT_HEADER, ACCEPT_HEADER_VALUE_SSZ).send().await?;
//     let response: GenericResponse<T> = request.json().await?;

//     Ok(response.data)
// }

impl BeaconApiClient {
    pub fn new(base_url: &String) -> Self {
        Self {
            http_client: Client::new(),
            base_url: base_url.to_string() + "/" + API_PREFIX
        }
    }
    
    pub async fn get_latest_headers(&self) -> ApiResult<BlockHeaderData> { 
        let endpoint = format!("{}/v1/beacon/headers", self.base_url);
        let result = get_call::<GenericResponse<Vec<BlockHeaderData>>>(&self.http_client, &endpoint).await?.data;
        let block_header_data = result.into_iter().nth(0);
        match block_header_data {
            Some(block_header_data) => Ok(block_header_data),
            None => Err(ApiClientError::ApiError("Error retrieving block header".to_string()))
        }
    }

    pub async fn get_latest_header(&self) -> ApiResult<BlockHeaderData> { 
        let endpoint = format!("{}/v1/beacon/headers/head", self.base_url);
        let block_header_data = get_call::<GenericResponse<BlockHeaderData>>(&self.http_client, &endpoint).await?.data;

        Ok(block_header_data)
    }

    pub async fn get_committees_at_state(&self, state_root: Hash256) -> ApiResult<Vec<CommitteeData>> {
        let endpoint = format!("{}/v1/beacon/states/{:#010x}/committees", self.base_url, &state_root);
        let committees = get_call::<GenericResponse<Vec<CommitteeData>>>(&self.http_client, &endpoint).await?.data;
        
        Ok(committees)
    }

    pub async fn get_sync_committees_at_state_root(&self, state_root: Hash256) -> ApiResult<SyncCommitteeByValidatorIndices> {
        let endpoint = format!("{}/v1/beacon/states/{:#010x}/sync_committees", self.base_url, &state_root);
        let committees = get_call::<GenericResponse<SyncCommitteeByValidatorIndices>>(&self.http_client, &endpoint).await?.data;
        
        Ok(committees)
    }
    
    pub async fn get_sync_committees_at_epoch(&self, state_root: Hash256, epoch: Epoch) -> ApiResult<SyncCommitteeByValidatorIndices> {
        let endpoint = format!("{}/v1/beacon/states/{:#010x}/sync_committees?{}", self.base_url, &state_root, &epoch.to_string());
        let committees = get_call::<GenericResponse<SyncCommitteeByValidatorIndices>>(&self.http_client, &endpoint).await?.data;
        
        Ok(committees)
    }

    pub async fn get_state_root_at_head(&self) -> ApiResult<RootData> {
        let endpoint = format!("{}/v1/beacon/states/head/root", self.base_url);
        let state_root = get_call::<GenericResponse<RootData>>(&self.http_client, &endpoint).await?.data;
        
        Ok(state_root)
    }

    pub async fn get_state_at_head(&self) -> ApiResult<BeaconState<MainnetEthSpec>> {
        let endpoint = format!("{}/v2/debug/beacon/states/head", self.base_url);
        let state = get_call::<GenericResponse<BeaconState<MainnetEthSpec>>>(&self.http_client, &endpoint).await?.data;
        
        Ok(state)
    }

    pub async fn get_signed_beacon_block<T: EthSpec>(&self, block_id: BlockId) -> ApiResult<ForkVersionedResponse<SignedBeaconBlock<T>>> {
        let endpoint = format!("{}/v2/beacon/blocks/{}", self.base_url, block_id.to_string());
        let signed_block = get_call::<ForkVersionedResponse<SignedBeaconBlock<T>>>(&self.http_client, &endpoint).await?;

        Ok(signed_block)
    }
   
    // pub async fn get_light_client_update(&self) -> ApiResult<LightClientUpdate> {
    //     // let endpoint = format!("{}/v1/lightclient/best_update/:periods", self.base_url);
    //     let header_data = self.get_latest_header().await?;

    //     let result: LightClientUpdate = LightClientUpdate {
    //         header: header_data.header.message,
    //         next_sync_committee: String::from("committee"),
    //         next_sync_committee_branch: vec!(Hash256::random()),
    //         finality_header: None,
    //         finality_branch: None,
    //         sync_committee_bits: vec![0,1,0,1],
    //         sync_committee_signature: String::from("signature"),
    //         fork_version: [1,2,3,4]
    //     };
    //     println!("{:#?}", result);
    //     Ok(result)
    // }
}

// pub async fn 