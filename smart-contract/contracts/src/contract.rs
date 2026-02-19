 use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::{storage, validation, Error, Origin, Product, TrackingEvent};

#[contract]
pub struct ChainLogisticsContract;

fn read_product(env: &Env, product_id: &String) -> Result<Product, Error> {
    storage::get_product(env, product_id).ok_or(Error::ProductNotFound)
}

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

fn require_can_add_event(env: &Env, product_id: &String, product: &Product, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
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

#[contractimpl]
impl ChainLogisticsContract {
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
        storage::set_auth(&env, &id, &owner, true);

        env.events().publish((Symbol::new(&env, "product_registered"), id.clone()), product.clone());
        Ok(product)
    }

    pub fn get_product(env: Env, id: String) -> Result<Product, Error> {
        read_product(&env, &id)
    }

    pub fn get_product_event_ids(env: Env, id: String) -> Result<Vec<u64>, Error> {
        let _ = read_product(&env, &id)?;
        Ok(storage::get_product_event_ids(&env, &id))
    }

    pub fn add_authorized_actor(env: Env, owner: Address, product_id: String, actor: Address) -> Result<(), Error> {
        let product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        storage::set_auth(&env, &product_id, &actor, true);
        Ok(())
    }

    pub fn remove_authorized_actor(env: Env, owner: Address, product_id: String, actor: Address) -> Result<(), Error> {
        let product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        storage::set_auth(&env, &product_id, &actor, false);
        Ok(())
    }

    pub fn transfer_product(env: Env, owner: Address, product_id: String, new_owner: Address) -> Result<(), Error> {
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
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        product.active = active;
        write_product(&env, &product);
        Ok(())
    }

    pub fn add_tracking_event(env: Env, actor: Address, product_id: String, event_type: Symbol, data_hash: BytesN<32>, note: String) -> Result<u64, Error> {
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
        Ok(event_id)
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
