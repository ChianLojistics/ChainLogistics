/// Tracking contract for managing product supply chain events.
/// This contract is responsible for:
/// - Adding tracking events to products
/// - Retrieving events by various criteria
/// - Managing event counts and statistics
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::error::Error;
use crate::storage;
use crate::types::{DataKey, TrackingEvent};
use crate::validation_contract::ValidationContract;
use crate::ChainLogisticsContractClient;

// ─── Storage helpers for TrackingContract ────────────────────────────────────

/// Get the main contract address from storage.
fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

/// Set the main contract address in storage.
fn set_main_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::MainContract, address);
}

/// Ensure the main contract is not paused.
/// Returns ContractPaused error if the main contract is paused.
fn require_not_paused(env: &Env) -> Result<(), Error> {
    let main_contract = get_main_contract(env).unwrap();
    let main_client = ChainLogisticsContractClient::new(env, &main_contract);
    if main_client.is_paused() {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

/// Ensure the tracking contract has been initialized.
/// Returns NotInitialized error if not initialized.
fn require_init(env: &Env) -> Result<(), Error> {
    if get_main_contract(env).is_none() {
        return Err(Error::NotInitialized);
    }
    Ok(())
}

// ─── Contract ────────────────────────────────────────────────────────────────

/// The Tracking contract manages supply chain events for products.
#[contract]
pub struct TrackingContract;

#[contractimpl]
impl TrackingContract {
    /// Initialize the TrackingContract with the main contract address.
    ///
    /// # Arguments
    /// * `main_contract` - The address of the main ChainLogistics contract
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns error if already initialized
    ///
    /// # Errors
    /// * `AlreadyInitialized` - If the contract has already been initialized
    pub fn init(env: Env, main_contract: Address) -> Result<(), Error> {
        if get_main_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        ValidationContract::validate_contract_address(&env, &main_contract)?;
        set_main_contract(&env, &main_contract);
        Ok(())
    }

    /// Add a new tracking event to a product.
    /// Requires authentication from the actor.
    /// Validates inputs and emits a tracking event.
    ///
    /// # Arguments
    /// * `actor` - The address adding the event (must be authorized)
    /// * `product_id` - The ID of the product
    /// * `event_type` - The type of event (e.g., "shipped", "received")
    /// * `location` - The location where the event occurred
    /// * `data_hash` - Hash of the event data
    /// * `note` - Optional note about the event
    /// * `metadata` - Additional metadata as key-value pairs
    ///
    /// # Returns
    /// * `Result<u64, Error>` - The ID of the newly created event
    ///
    /// # Errors
    /// * `NotInitialized` - If the tracking contract is not initialized
    /// * `ContractPaused` - If the main contract is paused
    /// * Various validation errors for invalid inputs
    pub fn tracking_add_event(
        env: Env,
        actor: Address,
        product_id: String,
        event_type: Symbol,
        location: String,
        data_hash: BytesN<32>,
        note: String,
        metadata: Map<Symbol, String>,
    ) -> Result<u64, Error> {
        require_init(&env)?;
        require_not_paused(&env)?;
        actor.require_auth();
        ValidationContract::non_empty(&product_id)?;
        ValidationContract::max_len(&product_id, ValidationContract::MAX_PRODUCT_ID_LEN)?;
        ValidationContract::validate_event_type(&env, &event_type)?;

        // Validate inputs early to fail fast and save gas
        ValidationContract::validate_event_location(&location)?;
        ValidationContract::validate_event_note(&note)?;
        ValidationContract::validate_metadata(&metadata)?;

        // Generate unique event ID (single storage read)
        let event_id = storage::next_event_id(&env)?;

        // Create event
        let event = TrackingEvent {
            event_id,
            product_id: product_id.clone(),
            actor,
            timestamp: env.ledger().timestamp(),
            event_type: event_type.clone(),
            location,
            data_hash,
            note,
            metadata,
        };

        // Batch storage operations for gas efficiency
        storage::put_event(&env, &event);

        // Update product event IDs (optimized: single read-modify-write)
        let mut ids = storage::get_product_event_ids(&env, &product_id);
        ids.push_back(event_id);
        storage::put_product_event_ids(&env, &product_id, &ids);

        // Index by type (single write)
        storage::index_event_by_type(&env, &product_id, &event_type, event_id)?;

        // Emit event (no storage cost)
        env.events().publish(
            (
                Symbol::new(&env, "tracking_event"),
                product_id.clone(),
                event_id,
            ),
            event.clone(),
        );

        Ok(event_id)
    }

    /// Get a single event by its ID.
    ///
    /// # Arguments
    /// * `event_id` - The ID of the event to retrieve
    ///
    /// # Returns
    /// * `Result<TrackingEvent, Error>` - The tracking event
    ///
    /// # Errors
    /// * `EventNotFound` - If the event does not exist
    pub fn tracking_get_event(env: Env, event_id: u64) -> Result<TrackingEvent, Error> {
        storage::get_event(&env, event_id).ok_or(Error::EventNotFound)
    }

    /// Get all event IDs for a product.
    ///
    /// # Arguments
    /// * `product_id` - The ID of the product
    ///
    /// # Returns
    /// * `Vec<u64>` - A vector of event IDs
    pub fn tracking_get_product_event_ids(env: Env, product_id: String) -> Vec<u64> {
        storage::get_product_event_ids(&env, &product_id)
    }

    /// Get the total event count for a product.
    ///
    /// # Arguments
    /// * `product_id` - The ID of the product
    ///
    /// # Returns
    /// * `u64` - The number of events
    pub fn tracking_get_event_count(env: Env, product_id: String) -> u64 {
        storage::get_product_event_ids(&env, &product_id).len() as u64
    }

    /// Get the count of events by type for a product.
    ///
    /// # Arguments
    /// * `product_id` - The ID of the product
    /// * `event_type` - The type of events to count
    ///
    /// # Returns
    /// * `u64` - The number of events of the specified type
    pub fn tracking_get_event_count_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
    ) -> u64 {
        storage::get_event_count_by_type(&env, &product_id, &event_type)
    }
}

#[cfg(test)]
mod test_tracking {
    use super::*;
    use crate::{
        AuthorizationContract, AuthorizationContractClient, ChainLogisticsContract,
        ChainLogisticsContractClient, ProductConfig, ProductRegistryContract,
        ProductRegistryContractClient,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, Map};

    fn setup_uninitialized(
        env: &Env,
    ) -> (
        ChainLogisticsContractClient,
        ProductRegistryContractClient,
        Address,
        Address,
        super::TrackingContractClient,
    ) {
        let auth_id = env.register_contract(None, AuthorizationContract);
        let cl_id = env.register_contract(None, ChainLogisticsContract);
        let registry_id = env.register_contract(None, ProductRegistryContract);
        let tracking_id = env.register_contract(None, super::TrackingContract);

        let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
        let registry_client = ProductRegistryContractClient::new(env, &registry_id);
        let tracking_client = super::TrackingContractClient::new(env, &tracking_id);

        let auth_client = AuthorizationContractClient::new(env, &auth_id);
        auth_client.configure_initializer(&registry_id);
        registry_client.configure_auth_contract(&auth_id);

        let admin = Address::generate(env);
        cl_client.init(&admin, &auth_id);

        (cl_client, registry_client, admin, cl_id, tracking_client)
    }

    fn setup_initialized(
        env: &Env,
    ) -> (
        ChainLogisticsContractClient,
        ProductRegistryContractClient,
        Address,
        Address,
        super::TrackingContractClient,
    ) {
        let (cl_client, registry_client, admin, cl_id, tracking_client) = setup_uninitialized(env);
        tracking_client.init(&cl_id);
        (cl_client, registry_client, admin, cl_id, tracking_client)
    }

    fn register_test_product(
        env: &Env,
        client: &ProductRegistryContractClient,
        owner: &Address,
        id: &str,
    ) -> String {
        let product_id = String::from_str(env, id);
        client.register_product(
            owner,
            &ProductConfig {
                id: product_id.clone(),
                name: String::from_str(env, "Test Product"),
                description: String::from_str(env, "Description"),
                origin_location: String::from_str(env, "Origin"),
                category: String::from_str(env, "Category"),
                tags: Vec::new(env),
                certifications: Vec::new(env),
                media_hashes: Vec::new(env),
                custom: Map::new(env),
            },
        );
        product_id
    }

    #[test]
    fn test_add_tracking_event() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, registry_client, _admin, _cl_id, tracking_client) =
            setup_initialized(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add tracking event
        let event_type = Symbol::new(&env, "created");
        let location = String::from_str(&env, "Warehouse A");
        let data_hash = BytesN::from_array(&env, &[0; 32]);
        let note = String::from_str(&env, "Product created");
        let metadata = Map::new(&env);

        let event_id = tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &note,
            &metadata,
        );

        // Verify event ID is sequential
        assert_eq!(event_id, 1);

        // Verify event count
        let count = tracking_client.tracking_get_event_count(&product_id);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_event() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, registry_client, _admin, _cl_id, tracking_client) =
            setup_initialized(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add tracking event
        let event_type = Symbol::new(&env, "created");
        let location = String::from_str(&env, "Warehouse A");
        let data_hash = BytesN::from_array(&env, &[0; 32]);
        let note = String::from_str(&env, "Product created");
        let metadata = Map::new(&env);

        let event_id = tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &note,
            &metadata,
        );

        // Get event
        let event = tracking_client.tracking_get_event(&event_id);
        assert_eq!(event.event_id, event_id);
        assert_eq!(event.product_id, product_id);
        assert_eq!(event.actor, owner);
        assert_eq!(event.event_type, event_type);
    }

    #[test]
    fn test_get_event_before_init_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, _registry_client, _admin, _cl_id, tracking_client) =
            setup_uninitialized(&env);

        // Get non-existent event
        let res = tracking_client.try_tracking_get_event(&999);
        assert_eq!(res, Err(Ok(Error::EventNotFound)));
    }

    #[test]
    fn test_add_event_when_paused_fails_and_unpause_restores() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, registry_client, admin, cl_id, tracking_client) = setup_initialized(&env);

        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        cl_client.pause(&admin);

        let event_type = Symbol::new(&env, "created");
        let location = String::from_str(&env, "Warehouse A");
        let data_hash = BytesN::from_array(&env, &[0; 32]);
        let note = String::from_str(&env, "Product created");
        let metadata = Map::new(&env);

        let res = tracking_client.try_tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &note,
            &metadata,
        );
        assert_eq!(res, Err(Ok(Error::ContractPaused)));

        cl_client.unpause(&admin);

        let event_id = tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &note,
            &metadata,
        );
        assert_eq!(event_id, 1);
    }

    #[test]
    fn test_add_multiple_events() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, registry_client, _admin, _cl_id, tracking_client) =
            setup_initialized(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add multiple events
        let event_type = Symbol::new(&env, "shipped");
        let location = String::from_str(&env, "Warehouse A");
        let data_hash = BytesN::from_array(&env, &[0; 32]);
        let metadata = Map::new(&env);

        let event_id1 = tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &String::from_str(&env, "First event"),
            &metadata,
        );

        let event_id2 = tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &String::from_str(&env, "Second event"),
            &metadata,
        );

        // Verify IDs are sequential
        assert_eq!(event_id1, 1);
        assert_eq!(event_id2, 2);

        // Verify event count
        let count = tracking_client.tracking_get_event_count(&product_id);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_product_event_ids() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, registry_client, _admin, _cl_id, tracking_client) =
            setup_initialized(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add events
        let event_type = Symbol::new(&env, "created");
        let location = String::from_str(&env, "Warehouse A");
        let data_hash = BytesN::from_array(&env, &[0; 32]);
        let metadata = Map::new(&env);

        tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &String::from_str(&env, "Event 1"),
            &metadata,
        );

        tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &String::from_str(&env, "Event 2"),
            &metadata,
        );

        // Get event IDs
        let event_ids = tracking_client.tracking_get_product_event_ids(&product_id);
        assert_eq!(event_ids.len(), 2);
        assert_eq!(event_ids.get(0), Some(1));
        assert_eq!(event_ids.get(1), Some(2));
    }

    #[test]
    fn test_event_count_by_type() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, registry_client, _admin, _cl_id, tracking_client) =
            setup_initialized(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        let location = String::from_str(&env, "Warehouse A");
        let data_hash = BytesN::from_array(&env, &[0; 32]);
        let metadata = Map::new(&env);

        // Add events of different types
        tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &Symbol::new(&env, "created"),
            &location,
            &data_hash,
            &String::from_str(&env, "Created"),
            &metadata,
        );

        tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &Symbol::new(&env, "shipped"),
            &location,
            &data_hash,
            &String::from_str(&env, "Shipped"),
            &metadata,
        );

        tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &Symbol::new(&env, "shipped"),
            &location,
            &data_hash,
            &String::from_str(&env, "Shipped again"),
            &metadata,
        );

        // Verify counts by type
        let created_count = tracking_client
            .tracking_get_event_count_by_type(&product_id, &Symbol::new(&env, "created"));
        let shipped_count = tracking_client
            .tracking_get_event_count_by_type(&product_id, &Symbol::new(&env, "shipped"));

        assert_eq!(created_count, 1);
        assert_eq!(shipped_count, 2);
    }

    #[test]
    fn test_add_event_with_metadata() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, registry_client, _admin, _cl_id, tracking_client) =
            setup_initialized(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add event with metadata
        let event_type = Symbol::new(&env, "created");
        let location = String::from_str(&env, "Warehouse A");
        let data_hash = BytesN::from_array(&env, &[0; 32]);

        let mut metadata = Map::new(&env);
        metadata.set(
            Symbol::new(&env, "temperature"),
            String::from_str(&env, "20C"),
        );
        metadata.set(Symbol::new(&env, "humidity"), String::from_str(&env, "50%"));

        let event_id = tracking_client.tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &String::from_str(&env, "With metadata"),
            &metadata,
        );

        // Verify event
        let event = tracking_client.tracking_get_event(&event_id);
        assert_eq!(event.metadata.len(), 2);
    }

    #[test]
    fn test_add_event_metadata_validation() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, registry_client, _admin, _cl_id, tracking_client) =
            setup_initialized(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Try to add event with too many metadata fields
        let mut metadata = Map::new(&env);
        // Add 25 fields (exceeds limit of 20) - using static keys
        metadata.set(Symbol::new(&env, "key0"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key1"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key2"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key3"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key4"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key5"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key6"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key7"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key8"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key9"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key10"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key11"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key12"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key13"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key14"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key15"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key16"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key17"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key18"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key19"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key20"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key21"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key22"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key23"), String::from_str(&env, "value"));
        metadata.set(Symbol::new(&env, "key24"), String::from_str(&env, "value"));

        let event_type = Symbol::new(&env, "created");
        let location = String::from_str(&env, "Warehouse A");
        let data_hash = BytesN::from_array(&env, &[0; 32]);

        let res = tracking_client.try_tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &String::from_str(&env, "Too much metadata"),
            &metadata,
        );

        assert_eq!(res, Err(Ok(Error::TooManyCustomFields)));
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, _registry_client, _admin, cl_id, tracking_client) =
            setup_uninitialized(&env);

        tracking_client.init(&cl_id);

        // Second init should fail
        let res = tracking_client.try_init(&cl_id);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_add_event_before_init_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let tracking_id = env.register_contract(None, super::TrackingContract);
        let tracking_client = super::TrackingContractClient::new(&env, &tracking_id);

        let owner = Address::generate(&env);
        let product_id = String::from_str(&env, "PROD1");
        let event_type = Symbol::new(&env, "created");
        let location = String::from_str(&env, "Warehouse A");
        let data_hash = BytesN::from_array(&env, &[0; 32]);
        let metadata = Map::new(&env);

        // Adding event without initialization should fail with NotInitialized
        let res = tracking_client.try_tracking_add_event(
            &owner,
            &product_id,
            &event_type,
            &location,
            &data_hash,
            &String::from_str(&env, "Test event"),
            &metadata,
        );

        assert_eq!(res, Err(Ok(Error::NotInitialized)));
    }
}
