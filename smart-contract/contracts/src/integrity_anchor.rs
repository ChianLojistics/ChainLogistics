#![allow(unexpected_cfgs)]
#![allow(dead_code)]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

use crate::error::Error;
use crate::events::{ContentAnchored, ContentTamperDetected};
use crate::storage;
use crate::types::{ContentAnchor, StorageScheme};
use crate::validation_contract::ValidationContract;
use crate::ProductRegistryContractClient;

/// Maximum file size: 50 MB
pub const MAX_CONTENT_SIZE: u64 = 52_428_800;

/// Maximum CID / storage reference length
pub const MAX_CID_LEN: u32 = 128;

/// Maximum anchors per product
pub const MAX_ANCHORS_PER_PRODUCT: u32 = 50;

#[contract]
pub struct IntegrityAnchorContract;

#[contractimpl]
impl IntegrityAnchorContract {
    /// Initialize with admin and the product registry contract address.
    pub fn init_integrity_anchor(
        env: Env,
        admin: Address,
        registry_contract: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        ValidationContract::validate_contract_address(&env, &registry_contract)?;
        storage::set_integrity_admin(&env, &admin);
        storage::set_integrity_registry(&env, &registry_contract);
        Ok(())
    }

    /// Anchor content by SHA-256 hash and decentralized storage CID.
    /// Identical content (CAS) reuses the existing anchor — no duplicate storage refs.
    pub fn anchor_content(
        env: Env,
        caller: Address,
        product_id: String,
        content_hash: BytesN<32>,
        cid: String,
        storage_scheme: StorageScheme,
        byte_size: u64,
    ) -> Result<ContentAnchor, Error> {
        caller.require_auth();

        ValidationContract::non_empty(&product_id)?;
        ValidationContract::non_empty(&cid)?;
        ValidationContract::max_len(&cid, MAX_CID_LEN)?;

        if byte_size == 0 || byte_size > MAX_CONTENT_SIZE {
            return Err(Error::InvalidContentSize);
        }

        let registry = storage::get_integrity_registry(&env).ok_or(Error::NotInitialized)?;
        let registry_client = ProductRegistryContractClient::new(&env, &registry);
        let _ = registry_client.get_product(&product_id);

        // CAS dedup: return existing anchor for identical content hash
        if let Some(existing_id) = storage::get_anchor_id_by_hash(&env, &content_hash) {
            if let Some(existing) = storage::get_content_anchor(&env, existing_id) {
                return Ok(existing);
            }
        }

        let anchor_id = storage::next_anchor_id(&env)?;
        let anchored_at = env.ledger().timestamp();

        let anchor = ContentAnchor {
            anchor_id,
            product_id: product_id.clone(),
            content_hash: content_hash.clone(),
            cid: cid.clone(),
            storage_scheme: storage_scheme.clone(),
            byte_size,
            anchored_at,
            anchored_by: caller.clone(),
        };

        storage::put_content_anchor(&env, &anchor);
        storage::set_anchor_id_by_hash(&env, &content_hash, anchor_id);
        storage::add_product_anchor(&env, &product_id, anchor_id)?;

        ContentAnchored {
            anchor_id,
            product_id: product_id.clone(),
            content_hash,
            cid,
            storage_scheme,
            byte_size,
            anchored_by: caller,
        }
        .publish(&env);

        Ok(anchor)
    }

    /// Retrieve a content anchor by ID.
    pub fn get_anchor(env: Env, anchor_id: u64) -> Result<ContentAnchor, Error> {
        storage::get_content_anchor(&env, anchor_id).ok_or(Error::AnchorNotFound)
    }

    /// List all content anchors for a product.
    pub fn get_anchors_for_product(env: Env, product_id: String) -> Result<Vec<ContentAnchor>, Error> {
        ValidationContract::non_empty(&product_id)?;
        Ok(storage::get_product_anchors(&env, &product_id))
    }

    /// On-chain integrity check: compares supplied hash against anchored hash.
    pub fn verify_anchor(
        env: Env,
        anchor_id: u64,
        content_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        let anchor = storage::get_content_anchor(&env, anchor_id).ok_or(Error::AnchorNotFound)?;
        let valid = anchor.content_hash == content_hash;

        if !valid {
            ContentTamperDetected {
                anchor_id,
                product_id: anchor.product_id,
                expected_hash: anchor.content_hash,
                supplied_hash: content_hash,
            }
            .publish(&env);
        }

        Ok(valid)
    }
}
