use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::{storage, validation, Error, EventIdPage, Origin, Product, ProductRegistrationInput, TrackingEvent, TrackingEventInput};

#[contract]
pub struct ChainLogisticsContract;

fn require_not_paused(env: &Env) -> Result<(), Error> {
    if storage::is_paused(env) {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin = storage::get_admin(env).ok_or(Error::NotInitialized)?;
    caller.require_auth();
    if &admin != caller {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

/// Reads a product from persistent storage.
/// 
/// Returns an error if the product doesn't exist.
fn read_product(env: &Env, product_id: &String) -> Result<Product, Error> {
    storage::get_product(env, product_id).ok_or(Error::ProductNotFound)
}

/// Writes a product to persistent storage.
/// 
/// Products are stored using Soroban's persistent storage API,
/// ensuring they persist across contract calls and ledger entries.
fn write_product(env: &Env, product: &Product) {
    storage::put_product(env, product);
}

fn require_owner(product: &Product, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    if &product.owner != caller {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

fn require_can_add_event_internal(
    env: &Env,
    product_id: &String,
    product: &Product,
    caller: &Address,
    require_auth: bool,
) -> Result<(), Error> {
    if require_auth {
        caller.require_auth();
    }
    if !product.active {
        return Err(Error::InvalidInput);
    }
    if &product.owner == caller {
        return Ok(());
    }
    if !storage::is_authorized(env, product_id, caller) {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

fn require_can_add_event(env: &Env, product_id: &String, product: &Product, caller: &Address) -> Result<(), Error> {
    require_can_add_event_internal(env, product_id, product, caller, true)
}

fn page_from_vec(env: &Env, ids: &Vec<u64>, cursor: u32, limit: u32) -> EventIdPage {
    if limit == 0 {
        return EventIdPage {
            ids: Vec::new(env),
            next_cursor: cursor,
        };
    }

    let len: u32 = ids.len();
    if cursor >= len {
        return EventIdPage {
            ids: Vec::new(env),
            next_cursor: len,
        };
    }

    let end = if cursor.saturating_add(limit) > len {
        len
    } else {
        cursor + limit
    };

    let mut out: Vec<u64> = Vec::new(env);
    for i in cursor..end {
        out.push_back(ids.get_unchecked(i));
    }

    EventIdPage {
        ids: out,
        next_cursor: end,
    }
}

fn page_recent_from_vec(env: &Env, ids: &Vec<u64>, cursor: u32, limit: u32) -> EventIdPage {
    if limit == 0 {
        return EventIdPage {
            ids: Vec::new(env),
            next_cursor: cursor,
        };
    }

    let len: u32 = ids.len();
    if cursor >= len {
        return EventIdPage {
            ids: Vec::new(env),
            next_cursor: len,
        };
    }

    let start_from_end = cursor;
    let remaining = len - start_from_end;
    let take = if limit > remaining { remaining } else { limit };

    let mut out: Vec<u64> = Vec::new(env);
    for j in 0..take {
        let idx = (len - 1) - (start_from_end + j);
        out.push_back(ids.get_unchecked(idx));
    }

    EventIdPage {
        ids: out,
        next_cursor: cursor + take,
    }
}

fn lower_bound(ts: &Vec<u64>, target: u64) -> u32 {
    let mut lo: u32 = 0;
    let mut hi: u32 = ts.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let v = ts.get_unchecked(mid);
        if v < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

fn upper_bound(ts: &Vec<u64>, target: u64) -> u32 {
    let mut lo: u32 = 0;
    let mut hi: u32 = ts.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let v = ts.get_unchecked(mid);
        if v <= target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

#[contractimpl]
impl ChainLogisticsContract {
    pub fn init(env: Env, admin: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_paused(&env, false);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env).ok_or(Error::NotInitialized)
    }

    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        storage::set_paused(&env, true);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        storage::set_paused(&env, false);
        Ok(())
    }

    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        new_admin.require_auth();
        storage::set_admin(&env, &new_admin);
        Ok(())
    }

    /// Registers a new product and stores it in persistent storage.
    /// Products persist across contract calls using Soroban's persistent storage API.
    /// Returns ProductAlreadyExists if the product ID already exists.
    pub fn register_product(
        env: Env,
        owner: Address,
        id: String,
        name: String,
        description: String,
        origin_location: String,
        category: String,
        tags: Vec<String>,
        certifications: Vec<BytesN<32>>,
        media_hashes: Vec<BytesN<32>>,
        custom: Map<Symbol, String>,
    ) -> Result<Product, Error> {
        require_not_paused(&env)?;
        const MAX_ID_LEN: u32 = 64;
        const MAX_NAME_LEN: u32 = 128;
        const MAX_ORIGIN_LEN: u32 = 256;
        const MAX_CATEGORY_LEN: u32 = 64;
        const MAX_DESCRIPTION_LEN: u32 = 2048;
        const MAX_TAG_LEN: u32 = 64;
        const MAX_CUSTOM_VALUE_LEN: u32 = 512;

        const MAX_TAGS: u32 = 20;
        const MAX_CERTIFICATIONS: u32 = 50;
        const MAX_MEDIA_HASHES: u32 = 50;
        const MAX_CUSTOM_FIELDS: u32 = 20;

        if !validation::non_empty(&id) {
            return Err(Error::InvalidProductId);
        }
        if !validation::max_len(&id, MAX_ID_LEN) {
            return Err(Error::ProductIdTooLong);
        }
        if !validation::non_empty(&name) {
            return Err(Error::InvalidProductName);
        }
        if !validation::max_len(&name, MAX_NAME_LEN) {
            return Err(Error::ProductNameTooLong);
        }
        if !validation::non_empty(&origin_location) {
            return Err(Error::InvalidOrigin);
        }
        if !validation::max_len(&origin_location, MAX_ORIGIN_LEN) {
            return Err(Error::OriginTooLong);
        }
        if !validation::non_empty(&category) {
            return Err(Error::InvalidCategory);
        }
        if !validation::max_len(&category, MAX_CATEGORY_LEN) {
            return Err(Error::CategoryTooLong);
        }
        if !validation::max_len(&description, MAX_DESCRIPTION_LEN) {
            return Err(Error::DescriptionTooLong);
        }

        if tags.len() > MAX_TAGS {
            return Err(Error::TooManyTags);
        }
        for i in 0..tags.len() {
            let t = tags.get_unchecked(i);
            if !validation::max_len(&t, MAX_TAG_LEN) {
                return Err(Error::TagTooLong);
            }
        }

        if certifications.len() > MAX_CERTIFICATIONS {
            return Err(Error::TooManyCertifications);
        }
        if media_hashes.len() > MAX_MEDIA_HASHES {
            return Err(Error::TooManyMediaHashes);
        }

        if custom.len() > MAX_CUSTOM_FIELDS {
            return Err(Error::TooManyCustomFields);
        }
        let custom_keys = custom.keys();
        for i in 0..custom_keys.len() {
            let k = custom_keys.get_unchecked(i);
            let v = custom.get_unchecked(k);
            if !validation::max_len(&v, MAX_CUSTOM_VALUE_LEN) {
                return Err(Error::CustomFieldValueTooLong);
            }
        }

        if storage::has_product(&env, &id) {
            return Err(Error::ProductAlreadyExists);
        }

        owner.require_auth();

        let product = Product {
            id: id.clone(),
            name,
            description,
            origin: Origin {
                location: origin_location,
            },
            owner: owner.clone(),
            created_at: env.ledger().timestamp(),
            active: true,
            category,
            tags,
            certifications,
            media_hashes,
            custom,
        };

        write_product(&env, &product);
        storage::put_product_event_ids(&env, &id, &Vec::new(&env));
        storage::put_product_event_timestamps(&env, &id, &Vec::new(&env));
        storage::set_auth(&env, &id, &owner, true);

        env.events().publish((Symbol::new(&env, "product_registered"), id.clone()), product.clone());
        Ok(product)
    }

    /// Retrieves a product from persistent storage by ID.
    /// Returns ProductNotFound if the product doesn't exist.
    pub fn get_product(env: Env, id: String) -> Result<Product, Error> {
        read_product(&env, &id)
    }

    pub fn get_product_event_ids(env: Env, id: String) -> Result<Vec<u64>, Error> {
        let _ = read_product(&env, &id)?;
        Ok(storage::get_product_event_ids(&env, &id))
    }

    pub fn get_product_event_ids_page(env: Env, id: String, cursor: u32, limit: u32) -> Result<EventIdPage, Error> {
        let _ = read_product(&env, &id)?;
        let ids = storage::get_product_event_ids(&env, &id);
        Ok(page_from_vec(&env, &ids, cursor, limit))
    }

    pub fn get_product_event_ids_rcnt_page(env: Env, id: String, cursor: u32, limit: u32) -> Result<EventIdPage, Error> {
        let _ = read_product(&env, &id)?;
        let ids = storage::get_product_event_ids(&env, &id);
        Ok(page_recent_from_vec(&env, &ids, cursor, limit))
    }

    pub fn get_evt_ids_type_page(env: Env, id: String, event_type: Symbol, cursor: u32, limit: u32) -> Result<EventIdPage, Error> {
        let _ = read_product(&env, &id)?;
        let ids = storage::get_product_event_ids_by_type(&env, &id, &event_type);
        Ok(page_from_vec(&env, &ids, cursor, limit))
    }

    pub fn get_evt_ids_actr_page(env: Env, id: String, actor: Address, cursor: u32, limit: u32) -> Result<EventIdPage, Error> {
        let _ = read_product(&env, &id)?;
        let ids = storage::get_product_event_ids_by_actor(&env, &id, &actor);
        Ok(page_from_vec(&env, &ids, cursor, limit))
    }

    pub fn get_evt_ids_date_page(
        env: Env,
        id: String,
        start_ts: u64,
        end_ts: u64,
        cursor: u32,
        limit: u32,
    ) -> Result<EventIdPage, Error> {
        let _ = read_product(&env, &id)?;
        if start_ts > end_ts {
            return Ok(EventIdPage {
                ids: Vec::new(&env),
                next_cursor: 0,
            });
        }

        let ts = storage::get_product_event_timestamps(&env, &id);
        let ids = storage::get_product_event_ids(&env, &id);
        if ts.len() != ids.len() {
            return Err(Error::InvalidInput);
        }

        let start_i = lower_bound(&ts, start_ts);
        let end_i = upper_bound(&ts, end_ts);
        if start_i >= end_i {
            return Ok(EventIdPage {
                ids: Vec::new(&env),
                next_cursor: 0,
            });
        }

        let range_len = end_i - start_i;
        if cursor >= range_len {
            return Ok(EventIdPage {
                ids: Vec::new(&env),
                next_cursor: range_len,
            });
        }

        let take = if limit == 0 {
            0
        } else if cursor.saturating_add(limit) > range_len {
            range_len - cursor
        } else {
            limit
        };

        let mut out: Vec<u64> = Vec::new(&env);
        for j in 0..take {
            out.push_back(ids.get_unchecked(start_i + cursor + j));
        }

        Ok(EventIdPage {
            ids: out,
            next_cursor: cursor + take,
        })
    }

    pub fn add_authorized_actor(env: Env, owner: Address, product_id: String, actor: Address) -> Result<(), Error> {
        require_not_paused(&env)?;
        let product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        storage::set_auth(&env, &product_id, &actor, true);
        Ok(())
    }

    pub fn remove_authorized_actor(env: Env, owner: Address, product_id: String, actor: Address) -> Result<(), Error> {
        require_not_paused(&env)?;
        let product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        storage::set_auth(&env, &product_id, &actor, false);
        Ok(())
    }

    pub fn transfer_product(env: Env, owner: Address, product_id: String, new_owner: Address) -> Result<(), Error> {
        require_not_paused(&env)?;
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;

        new_owner.require_auth();

        storage::set_auth(&env, &product_id, &product.owner, false);
        product.owner = new_owner.clone();
        write_product(&env, &product);
        storage::set_auth(&env, &product_id, &new_owner, true);
        Ok(())
    }

    pub fn set_product_active(env: Env, owner: Address, product_id: String, active: bool) -> Result<(), Error> {
        require_not_paused(&env)?;
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        product.active = active;
        write_product(&env, &product);
        Ok(())
    }

    pub fn add_tracking_event(env: Env, actor: Address, product_id: String, event_type: Symbol, data_hash: BytesN<32>, note: String) -> Result<u64, Error> {
        require_not_paused(&env)?;
        let product = read_product(&env, &product_id)?;
        require_can_add_event(&env, &product_id, &product, &actor)?;

        let event_id = storage::next_event_id(&env);
        let event = TrackingEvent {
            event_id,
            product_id: product_id.clone(),
            actor,
            timestamp: env.ledger().timestamp(),
            event_type,
            data_hash,
            note,
        };

        storage::put_event(&env, &event);
        let mut ids = storage::get_product_event_ids(&env, &product_id);
        ids.push_back(event_id);
        storage::put_product_event_ids(&env, &product_id, &ids);

        let mut ts = storage::get_product_event_timestamps(&env, &product_id);
        ts.push_back(event.timestamp);
        storage::put_product_event_timestamps(&env, &product_id, &ts);

        let mut by_type = storage::get_product_event_ids_by_type(&env, &product_id, &event.event_type);
        by_type.push_back(event_id);
        storage::put_product_event_ids_by_type(&env, &product_id, &event.event_type, &by_type);

        let mut by_actor = storage::get_product_event_ids_by_actor(&env, &product_id, &event.actor);
        by_actor.push_back(event_id);
        storage::put_product_event_ids_by_actor(&env, &product_id, &event.actor, &by_actor);
        Ok(event_id)
    }

    pub fn register_products_batch(env: Env, owner: Address, inputs: Vec<ProductRegistrationInput>) -> Result<Vec<Product>, Error> {
        require_not_paused(&env)?;
        const MAX_BATCH: u32 = 100;

        let n = inputs.len();
        if n == 0 {
            return Err(Error::EmptyBatch);
        }
        if n > MAX_BATCH {
            return Err(Error::BatchTooLarge);
        }

        owner.require_auth();

        // Pre-validate everything (including duplicates within the batch) before any writes.
        let mut seen: Map<String, bool> = Map::new(&env);
        for i in 0..n {
            let inp = inputs.get_unchecked(i);

            // Duplicate id check within batch
            if seen.get(inp.id.clone()).unwrap_or(false) {
                return Err(Error::DuplicateProductIdInBatch);
            }
            seen.set(inp.id.clone(), true);

            // Reuse existing single-item validation by calling it inline.
            // Keep limits consistent with register_product.
            const MAX_ID_LEN: u32 = 64;
            const MAX_NAME_LEN: u32 = 128;
            const MAX_ORIGIN_LEN: u32 = 256;
            const MAX_CATEGORY_LEN: u32 = 64;
            const MAX_DESCRIPTION_LEN: u32 = 2048;
            const MAX_TAG_LEN: u32 = 64;
            const MAX_CUSTOM_VALUE_LEN: u32 = 512;

            const MAX_TAGS: u32 = 20;
            const MAX_CERTIFICATIONS: u32 = 50;
            const MAX_MEDIA_HASHES: u32 = 50;
            const MAX_CUSTOM_FIELDS: u32 = 20;

            if !validation::non_empty(&inp.id) {
                return Err(Error::InvalidProductId);
            }
            if !validation::max_len(&inp.id, MAX_ID_LEN) {
                return Err(Error::ProductIdTooLong);
            }
            if !validation::non_empty(&inp.name) {
                return Err(Error::InvalidProductName);
            }
            if !validation::max_len(&inp.name, MAX_NAME_LEN) {
                return Err(Error::ProductNameTooLong);
            }
            if !validation::non_empty(&inp.origin_location) {
                return Err(Error::InvalidOrigin);
            }
            if !validation::max_len(&inp.origin_location, MAX_ORIGIN_LEN) {
                return Err(Error::OriginTooLong);
            }
            if !validation::non_empty(&inp.category) {
                return Err(Error::InvalidCategory);
            }
            if !validation::max_len(&inp.category, MAX_CATEGORY_LEN) {
                return Err(Error::CategoryTooLong);
            }
            if !validation::max_len(&inp.description, MAX_DESCRIPTION_LEN) {
                return Err(Error::DescriptionTooLong);
            }

            if inp.tags.len() > MAX_TAGS {
                return Err(Error::TooManyTags);
            }
            for j in 0..inp.tags.len() {
                let t = inp.tags.get_unchecked(j);
                if !validation::max_len(&t, MAX_TAG_LEN) {
                    return Err(Error::TagTooLong);
                }
            }
            if inp.certifications.len() > MAX_CERTIFICATIONS {
                return Err(Error::TooManyCertifications);
            }
            if inp.media_hashes.len() > MAX_MEDIA_HASHES {
                return Err(Error::TooManyMediaHashes);
            }

            if inp.custom.len() > MAX_CUSTOM_FIELDS {
                return Err(Error::TooManyCustomFields);
            }
            let custom_keys = inp.custom.keys();
            for j in 0..custom_keys.len() {
                let k = custom_keys.get_unchecked(j);
                let v = inp.custom.get_unchecked(k);
                if !validation::max_len(&v, MAX_CUSTOM_VALUE_LEN) {
                    return Err(Error::CustomFieldValueTooLong);
                }
            }

            if storage::has_product(&env, &inp.id) {
                return Err(Error::ProductAlreadyExists);
            }
        }

        // Execute writes
        let mut products: Vec<Product> = Vec::new(&env);
        for i in 0..n {
            let inp = inputs.get_unchecked(i);
            let product = Product {
                id: inp.id.clone(),
                name: inp.name.clone(),
                description: inp.description.clone(),
                origin: Origin {
                    location: inp.origin_location.clone(),
                },
                owner: owner.clone(),
                created_at: env.ledger().timestamp(),
                active: true,
                category: inp.category.clone(),
                tags: inp.tags.clone(),
                certifications: inp.certifications.clone(),
                media_hashes: inp.media_hashes.clone(),
                custom: inp.custom.clone(),
            };

            write_product(&env, &product);
            storage::put_product_event_ids(&env, &product.id, &Vec::new(&env));
            storage::put_product_event_timestamps(&env, &product.id, &Vec::new(&env));
            storage::set_auth(&env, &product.id, &owner, true);
            env.events().publish((Symbol::new(&env, "product_registered"), product.id.clone()), product.clone());
            products.push_back(product);
        }

        Ok(products)
    }

    pub fn add_tracking_events_batch(env: Env, actor: Address, inputs: Vec<TrackingEventInput>) -> Result<Vec<u64>, Error> {
        require_not_paused(&env)?;
        const MAX_BATCH: u32 = 200;

        let n = inputs.len();
        if n == 0 {
            return Err(Error::EmptyBatch);
        }
        if n > MAX_BATCH {
            return Err(Error::BatchTooLarge);
        }

        actor.require_auth();

        // Pre-validate first (no writes). Auth already required above.
        for i in 0..n {
            let inp = inputs.get(i).unwrap();
            let product = read_product(&env, &inp.product_id)?;
            require_can_add_event_internal(&env, &inp.product_id, &product, &actor, false)?;
        }

        let now = env.ledger().timestamp();

        let mut event_ids: Vec<u64> = Vec::new(&env);
        let mut per_product_ids: Map<String, Vec<u64>> = Map::new(&env);
        let mut per_product_ts: Map<String, Vec<u64>> = Map::new(&env);
        let mut per_product_type: Map<(String, Symbol), Vec<u64>> = Map::new(&env);
        let mut per_product_actor: Map<(String, Address), Vec<u64>> = Map::new(&env);

        for i in 0..n {
            let inp = inputs.get(i).unwrap();
            let event_id = storage::next_event_id(&env);
            let event = TrackingEvent {
                event_id,
                product_id: inp.product_id.clone(),
                actor: actor.clone(),
                timestamp: now,
                event_type: inp.event_type.clone(),
                data_hash: inp.data_hash.clone(),
                note: inp.note.clone(),
            };

            storage::put_event(&env, &event);
            event_ids.push_back(event_id);

            // Timeline IDs
            let mut ids = per_product_ids
                .get(inp.product_id.clone())
                .unwrap_or(storage::get_product_event_ids(&env, &inp.product_id));
            ids.push_back(event_id);
            per_product_ids.set(inp.product_id.clone(), ids);

            // Timestamps aligned with timeline
            let mut ts = per_product_ts
                .get(inp.product_id.clone())
                .unwrap_or(storage::get_product_event_timestamps(&env, &inp.product_id));
            ts.push_back(now);
            per_product_ts.set(inp.product_id.clone(), ts);

            // By type
            let k_type = (inp.product_id.clone(), inp.event_type.clone());
            let mut by_type = per_product_type
                .get(k_type.clone())
                .unwrap_or(storage::get_product_event_ids_by_type(&env, &inp.product_id, &inp.event_type));
            by_type.push_back(event_id);
            per_product_type.set(k_type, by_type);

            // By actor
            let k_actor = (inp.product_id.clone(), actor.clone());
            let mut by_actor = per_product_actor
                .get(k_actor.clone())
                .unwrap_or(storage::get_product_event_ids_by_actor(&env, &inp.product_id, &actor));
            by_actor.push_back(event_id);
            per_product_actor.set(k_actor, by_actor);
        }

        let product_keys = per_product_ids.keys();
        for i in 0..product_keys.len() {
            let pid = product_keys.get_unchecked(i);
            let ids = per_product_ids.get_unchecked(pid.clone());
            storage::put_product_event_ids(&env, &pid, &ids);
        }

        let ts_keys = per_product_ts.keys();
        for i in 0..ts_keys.len() {
            let pid = ts_keys.get_unchecked(i);
            let ts = per_product_ts.get_unchecked(pid.clone());
            storage::put_product_event_timestamps(&env, &pid, &ts);
        }

        let type_keys = per_product_type.keys();
        for i in 0..type_keys.len() {
            let k = type_keys.get_unchecked(i);
            let ids = per_product_type.get_unchecked(k.clone());
            storage::put_product_event_ids_by_type(&env, &k.0, &k.1, &ids);
        }

        let actor_keys = per_product_actor.keys();
        for i in 0..actor_keys.len() {
            let k = actor_keys.get_unchecked(i);
            let ids = per_product_actor.get_unchecked(k.clone());
            storage::put_product_event_ids_by_actor(&env, &k.0, &k.1, &ids);
        }

        Ok(event_ids)
    }

    pub fn get_event(env: Env, event_id: u64) -> Result<TrackingEvent, Error> {
        storage::get_event(&env, event_id).ok_or(Error::EventNotFound)
    }

    pub fn is_authorized(env: Env, product_id: String, actor: Address) -> Result<bool, Error> {
        let product = read_product(&env, &product_id)?;
        if product.owner == actor {
            return Ok(true);
        }
        Ok(storage::is_authorized(&env, &product_id, &actor))
    }
}
