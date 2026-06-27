//! On-chain integrity anchors for decentralized content (manuals, PDFs, media).
//!
//! Content is stored directly on IPFS or Arweave — this contract only records the
//! content-addressed hash (SHA-256) and CID/transaction id so tamper detection can
//! compare fetched bytes against the anchor. Identical manuals deduplicate via CAS:
//! anchoring the same `content_hash` twice is idempotent.

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::events::{IntegrityAnchored, IntegrityTamperFlagged};
use crate::types::{ContentAnchor, DataKey};
use crate::validation_contract::ValidationContract;
use crate::ChainLogisticsContractClient;

/// Maximum anchored file size (50 MiB). Enforced off-chain as well; on-chain guard
/// prevents oversized metadata from being registered.
pub const MAX_ANCHOR_FILE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_CID_LEN: u32 = 128;

fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

fn require_init(env: &Env) -> Result<(), Error> {
    if get_main_contract(env).is_none() {
        return Err(Error::NotInitialized);
    }
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), Error> {
    let main_contract = get_main_contract(env).ok_or(Error::NotInitialized)?;
    let main_client = ChainLogisticsContractClient::new(env, &main_contract);
    if main_client.is_paused() {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

fn get_anchor(env: &Env, content_hash: &BytesN<32>) -> Option<ContentAnchor> {
    env.storage()
        .persistent()
        .get(&DataKey::ContentAnchor(content_hash.clone()))
}

fn put_anchor(env: &Env, anchor: &ContentAnchor) {
    env.storage()
        .persistent()
        .set(&DataKey::ContentAnchor(anchor.content_hash.clone()), anchor);
}

fn append_product_anchor(env: &Env, product_id: &String, content_hash: &BytesN<32>) {
    let mut hashes: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&DataKey::ProductContentAnchors(product_id.clone()))
        .unwrap_or_else(|| Vec::new(env));

    let mut found = false;
    for i in 0..hashes.len() {
        if hashes.get(i).unwrap() == *content_hash {
            found = true;
            break;
        }
    }
    if !found {
        hashes.push_back(content_hash.clone());
        env.storage()
            .persistent()
            .set(&DataKey::ProductContentAnchors(product_id.clone()), &hashes);
    }
}

fn validate_backend_env(env: &Env, backend: &Symbol) -> Result<(), Error> {
    let ipfs = Symbol::new(env, "ipfs");
    let arweave = Symbol::new(env, "arweave");
    if backend != &ipfs && backend != &arweave {
        return Err(Error::InvalidStorageBackend);
    }
    Ok(())
}

#[contract]
pub struct IntegrityAnchorContract;

#[contractimpl]
impl IntegrityAnchorContract {
    /// Wire this satellite contract to the main ChainLogistics contract.
    pub fn init(env: Env, main_contract: Address) {
        ValidationContract::validate_contract_address(&env, &main_contract)
            .unwrap_or_else(|_| panic!("invalid main contract"));
        env.storage()
            .persistent()
            .set(&DataKey::MainContract, &main_contract);
    }

    /// Anchor a content hash and its decentralized storage CID.
    ///
    /// CAS deduplication: if `content_hash` is already anchored with the same CID,
    /// this call succeeds without duplicating state.
    pub fn anchor_content(
        env: Env,
        anchorer: Address,
        content_hash: BytesN<32>,
        cid: String,
        backend: Symbol,
        product_id: String,
        byte_size: u64,
    ) -> Result<(), Error> {
        require_init(&env)?;
        require_not_paused(&env)?;
        anchorer.require_auth();

        ValidationContract::non_empty(&cid)?;
        ValidationContract::max_len(&cid, MAX_CID_LEN)?;
        ValidationContract::non_empty(&product_id)?;
        ValidationContract::max_len(&product_id, ValidationContract::MAX_PRODUCT_ID_LEN)?;
        validate_backend_env(&env, &backend)?;

        if byte_size == 0 || byte_size > MAX_ANCHOR_FILE_BYTES {
            return Err(Error::FileTooLarge);
        }

        if let Some(existing) = get_anchor(&env, &content_hash) {
            if existing.cid == cid && existing.backend == backend {
                return Ok(());
            }
            return Err(Error::AnchorHashMismatch);
        }

        let anchor = ContentAnchor {
            content_hash: content_hash.clone(),
            cid: cid.clone(),
            backend: backend.clone(),
            product_id: product_id.clone(),
            byte_size,
            anchored_at: env.ledger().timestamp(),
            anchored_by: anchorer.clone(),
            tamper_detected: false,
        };

        put_anchor(&env, &anchor);
        append_product_anchor(&env, &product_id, &content_hash);

        IntegrityAnchored {
            content_hash,
            cid,
            backend,
            product_id,
            byte_size,
            anchorer,
        }
        .publish(&env);

        Ok(())
    }

    /// Return the on-chain anchor for a content hash, if present.
    pub fn get_anchor(env: Env, content_hash: BytesN<32>) -> Result<ContentAnchor, Error> {
        require_init(&env)?;
        get_anchor(&env, &content_hash).ok_or(Error::AnchorNotFound)
    }

    /// Whether a content hash has been anchored (CAS lookup).
    pub fn is_anchored(env: Env, content_hash: BytesN<32>) -> Result<bool, Error> {
        require_init(&env)?;
        Ok(get_anchor(&env, &content_hash).is_some())
    }

    /// List content hashes anchored for a product.
    pub fn get_product_anchors(env: Env, product_id: String) -> Result<Vec<BytesN<32>>, Error> {
        require_init(&env)?;
        ValidationContract::non_empty(&product_id)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::ProductContentAnchors(product_id))
            .unwrap_or_else(|| Vec::new(&env)))
    }

    /// Record that off-chain verification detected tampering.
    pub fn flag_tamper(
        env: Env,
        reporter: Address,
        content_hash: BytesN<32>,
    ) -> Result<(), Error> {
        require_init(&env)?;
        reporter.require_auth();

        let mut anchor = get_anchor(&env, &content_hash).ok_or(Error::AnchorNotFound)?;
        if anchor.tamper_detected {
            return Ok(());
        }

        anchor.tamper_detected = true;
        put_anchor(&env, &anchor);

        IntegrityTamperFlagged {
            content_hash: content_hash.clone(),
            cid: anchor.cid.clone(),
            product_id: anchor.product_id.clone(),
            reporter,
        }
        .publish(&env);

        Ok(())
    }
}

#[cfg(test)]
mod test_integrity_anchor {
    use super::*;
    use crate::{ChainLogisticsContract, ChainLogisticsContractClient};
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Symbol};

    fn setup(env: &Env) -> (IntegrityAnchorContractClient<'static>, Address) {
        let cl_id = env.register_contract(None, ChainLogisticsContract);
        let ia_id = env.register_contract(None, IntegrityAnchorContract);
        let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
        let ia_client = IntegrityAnchorContractClient::new(env, &ia_id);

        let admin = Address::generate(env);
        let auth_contract = Address::generate(env);
        cl_client.init(&admin, &auth_contract);
        ia_client.init(&cl_id);
        (ia_client, admin)
    }

    fn sample_hash(env: &Env, byte: u8) -> BytesN<32> {
        BytesN::from_array(env, &[byte; 32])
    }

    #[test]
    fn anchor_and_cas_dedup() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, anchorer) = setup(&env);

        let hash = sample_hash(&env, 1);
        let cid = String::from_str(&env, "QmTest123");
        let backend = Symbol::new(&env, "ipfs");
        let product_id = String::from_str(&env, "PROD-001");

        client.anchor_content(
            &anchorer,
            &hash,
            &cid,
            &backend,
            &product_id,
            &1024,
        );

        assert!(client.is_anchored(&hash));
        let anchor = client.get_anchor(&hash);
        assert_eq!(anchor.cid, cid);
        assert_eq!(anchor.byte_size, 1024);

        // Idempotent re-anchor with same hash + CID
        client.anchor_content(
            &anchorer,
            &hash,
            &cid,
            &backend,
            &product_id,
            &1024,
        );

        let anchors = client.get_product_anchors(&product_id);
        assert_eq!(anchors.len(), 1);
    }

    #[test]
    fn rejects_oversized_file_metadata() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, anchorer) = setup(&env);

        let hash = sample_hash(&env, 2);
        let cid = String::from_str(&env, "QmBig");
        let backend = Symbol::new(&env, "ipfs");
        let product_id = String::from_str(&env, "PROD-002");

        let result = client.try_anchor_content(
            &anchorer,
            &hash,
            &cid,
            &backend,
            &product_id,
            &(MAX_ANCHOR_FILE_BYTES + 1),
        );
        assert_eq!(result, Err(Ok(Error::FileTooLarge)));
    }

    #[test]
    fn flag_tamper_sets_state() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, anchorer) = setup(&env);

        let hash = sample_hash(&env, 3);
        let cid = String::from_str(&env, "QmTamper");
        let backend = Symbol::new(&env, "arweave");
        let product_id = String::from_str(&env, "PROD-003");

        client.anchor_content(
            &anchorer,
            &hash,
            &cid,
            &backend,
            &product_id,
            &4096,
        );

        client.flag_tamper(&anchorer, &hash);
        let anchor = client.get_anchor(&hash);
        assert!(anchor.tamper_detected);
    }
}
