use std::{str::FromStr, sync::Arc};

use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

use crate::AppState;

#[derive(Debug, BorshDeserialize)]
pub struct Satellite {
    pub owner: Pubkey,
    // For #[max_len(30)] String, Borsh serializes as 4 bytes (length) + 30 bytes (data/padding) = 34 bytes
    pub name: [u8; 34],    // Use fixed-size byte array to match serialized length
    pub country: [u8; 34], // Same for country
    pub norad_id: u64,     // Corrected to u64, matching your program's definition
    pub launch_date: i64,
    pub mint_date: i64,
    pub orbit_type: [u8; 34], // Same for orbit_type
    pub inclination: f64,
    pub altitude: f64,
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub raan: f64,
    pub arg_of_periapsis: f64,
    pub maneuver_type: ManeuverType,
    pub operation_status: OperationStatus,
}

impl Satellite {
    // Helper to extract the actual String from the fixed-size byte array
    // This correctly reads the u32 length prefix, then the bytes.
    pub fn get_string_from_padded_bytes(bytes: &[u8]) -> String {
        // Ensure bytes has at least 4 bytes for the length prefix
        if bytes.len() < 4 {
            return String::from(""); // Or handle error appropriately
        }
        let len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        // Ensure we don't go out of bounds if `len` is larger than remaining bytes
        let data_end = 4 + len;
        if data_end > bytes.len() {
            return String::from_utf8_lossy(&bytes[4..]).to_string(); // Fallback if length is corrupted
        }
        String::from_utf8_lossy(&bytes[4..data_end]).to_string()
    }

    pub fn name_as_string(&self) -> String {
        Self::get_string_from_padded_bytes(&self.name)
    }

    pub fn country_as_string(&self) -> String {
        Self::get_string_from_padded_bytes(&self.country)
    }

    pub fn orbit_type_as_string(&self) -> String {
        Self::get_string_from_padded_bytes(&self.orbit_type)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, BorshDeserialize)]
pub enum OperationStatus {
    Active,
    Maintenance,
    Offline,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, BorshDeserialize)]
pub enum ManeuverType {
    StationKeeping,
    OrbitRaising,
    OrbitLowering,
    InclinationChange,
    PhaseAdjustment,
    CollisionAvoidance,
    EndOfLife,
    Desaturation,
}

#[derive(Debug, serde::Serialize, Clone)] // This one retains `serde::Serialize`
pub struct SatelliteApiResponse {
    pub owner: Pubkey,
    pub name: String,
    pub country: String,
    pub norad_id: u64,
    pub launch_date: i64,
    pub mint_date: i64,
    pub orbit_type: String,
    pub inclination: f64,
    pub altitude: f64,
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub raan: f64,
    pub arg_of_periapsis: f64,
    pub maneuver_type: ManeuverType,
    pub operation_status: OperationStatus,
}

impl From<Satellite> for SatelliteApiResponse {
    fn from(account: Satellite) -> Self {
        SatelliteApiResponse {
            owner: account.owner,
            name: account.name_as_string(), // Call the helper to get clean string
            country: account.country_as_string(), // Call the helper
            norad_id: account.norad_id,
            launch_date: account.launch_date,
            mint_date: account.mint_date,
            orbit_type: account.orbit_type_as_string(), // Call the helper
            inclination: account.inclination,
            altitude: account.altitude,
            semi_major_axis: account.semi_major_axis,
            eccentricity: account.eccentricity,
            raan: account.raan,
            arg_of_periapsis: account.arg_of_periapsis,
            maneuver_type: account.maneuver_type,
            operation_status: account.operation_status,
        }
    }
}

#[debug_handler]
pub async fn get_satellite_from_norad_id(
    Path((user_authority_str, registry_authority_str, norad_id_str)): Path<(
        String,
        String,
        String,
    )>,
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<SatelliteApiResponse>, StatusCode> {
    // Parse the string parameters into their correct types
    let user_authority_pubkey = Pubkey::from_str(&user_authority_str).map_err(|e| {
        eprintln!(
            "Invalid user authority Pubkey: {}. Error: {}",
            user_authority_str, e
        );
        StatusCode::BAD_REQUEST
    })?;

    let registry_authority_pubkey = Pubkey::from_str(&registry_authority_str).map_err(|e| {
        eprintln!(
            "Invalid registry authority Pubkey: {}. Error: {}",
            registry_authority_str, e
        );
        StatusCode::BAD_REQUEST
    })?;

    let norad_id: u64 = norad_id_str.parse().map_err(|e| {
        eprintln!("Invalid NORAD ID: {}. Error: {}", norad_id_str, e);
        StatusCode::BAD_REQUEST
    })?;

    // derive pda
    let (pda_pubkey, _bump) = Pubkey::find_program_address(
        &[
            b"satellite",
            user_authority_pubkey.as_ref(),
            registry_authority_pubkey.as_ref(),
            &norad_id.to_le_bytes(), // Convert u32 to little-endian bytes
        ],
        &app_state.program_id, // Use the program ID from your AppState
    );

    println!("Derived Satellite PDA Pubkey: {}", pda_pubkey);

    // fetch account details
    let account = app_state.rpc_client.get_account(&pda_pubkey);

    match account {
        Ok(account) => {
            // 3. Deserialize the account data using Borsh
            // Be very careful that SatelliteProgramAccount exactly matches the on-chain layout.
            let data_slice = &account.data[8..];
            let satellite_data: Satellite =
                BorshDeserialize::try_from_slice(data_slice).map_err(|e| {
                    eprintln!(
                        "Failed to deserialize Satellite account data for PDA {}: {:?}",
                        pda_pubkey, e
                    );
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            println!(
                "Successfully deserialized Satellite data: {:?}",
                satellite_data
            );
            Ok(Json(satellite_data.into()))
        }
        Err(e) => {
            eprintln!(
                "Error fetching account data for Satellite PDA {}: {:?}",
                pda_pubkey, e
            );
            // Handle different RPC errors:
            // Check if the error indicates the account was not found
            if e.to_string().contains("AccountNotFound") {
                Err(StatusCode::NOT_FOUND) // Return 404 if account not found
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR) // Generic 500 for other RPC errors
            }
        }
    }
}
