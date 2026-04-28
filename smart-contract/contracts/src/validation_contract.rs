#![allow(clippy::len_zero)]
use soroban_sdk::{Address, Env, Map, String, Symbol};

use crate::error::Error;
use crate::types::ProductConfig;

pub struct ValidationContract;

impl ValidationContract {
    // --- String length limits ---
    pub const MAX_PRODUCT_ID_LEN: u32 = 64;
    pub const MAX_PRODUCT_NAME_LEN: u32 = 128;
    pub const MAX_ORIGIN_LEN: u32 = 128;
    pub const MAX_CATEGORY_LEN: u32 = 64;
    pub const MAX_DESCRIPTION_LEN: u32 = 512;
    pub const MAX_TAG_LEN: u32 = 64;
    pub const MAX_CUSTOM_VALUE_LEN: u32 = 256;
    pub const MAX_NOTE_LEN: u32 = 512;
    pub const MAX_LOCATION_LEN: u32 = 128;

    // --- Array / collection size limits ---
    pub const MAX_TAGS: u32 = 20;
    pub const MAX_CERTIFICATIONS: u32 = 50;
    pub const MAX_MEDIA_HASHES: u32 = 50;
    pub const MAX_CUSTOM_FIELDS: u32 = 20;
    pub const MAX_METADATA_FIELDS: u32 = 20;
    pub const MAX_PAGE_LIMIT: u64 = 1_000;

    // --- Primitive validators ---
    pub fn non_empty(s: &String) -> Result<(), Error> {
        if s.len() < 1 {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }

    pub fn max_len(s: &String, max: u32) -> Result<(), Error> {
        if s.len() > max {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }

    pub fn require_auth(actor: &Address) -> Result<(), Error> {
        actor.require_auth();
        Ok(())
    }

    pub fn validate_contract_address(env: &Env, address: &Address) -> Result<(), Error> {
        if *address == env.current_contract_address() {
            return Err(Error::InvalidAddress);
        }
        Ok(())
    }

    pub fn validate_distinct_addresses(a: &Address, b: &Address) -> Result<(), Error> {
        if a == b {
            return Err(Error::InvalidAddress);
        }
        Ok(())
    }

    pub fn validate_event_type(env: &Env, event_type: &Symbol) -> Result<(), Error> {
        if *event_type == Symbol::new(env, "") {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }

    pub fn validate_pagination_limit(limit: u64, max: u64) -> Result<(), Error> {
        if limit == 0 || limit > max {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }

    pub fn validate_time_range(env: &Env, start_time: u64, end_time: u64) -> Result<(), Error> {
        let now = env.ledger().timestamp();
        if end_time == u64::MAX {
            if start_time > now {
                return Err(Error::InvalidTimestamp);
            }
            return Ok(());
        }
        if start_time > end_time || end_time > now {
            return Err(Error::InvalidTimestamp);
        }
        Ok(())
    }

    // --- Product validation ---
    pub fn validate_product_config(config: &ProductConfig) -> Result<(), Error> {
        if config.id.len() < 1 {
            return Err(Error::InvalidProductId);
        }
        if config.id.len() > Self::MAX_PRODUCT_ID_LEN {
            return Err(Error::ProductIdTooLong);
        }

        if config.name.len() < 1 {
            return Err(Error::InvalidProductName);
        }
        if config.name.len() > Self::MAX_PRODUCT_NAME_LEN {
            return Err(Error::ProductNameTooLong);
        }

        if config.origin_location.len() < 1 {
            return Err(Error::InvalidOrigin);
        }
        if config.origin_location.len() > Self::MAX_ORIGIN_LEN {
            return Err(Error::OriginTooLong);
        }

        if config.category.len() < 1 {
            return Err(Error::InvalidCategory);
        }
        if config.category.len() > Self::MAX_CATEGORY_LEN {
            return Err(Error::CategoryTooLong);
        }

        if config.description.len() > Self::MAX_DESCRIPTION_LEN {
            return Err(Error::DescriptionTooLong);
        }

        if config.tags.len() > Self::MAX_TAGS {
            return Err(Error::TooManyTags);
        }
        for i in 0..config.tags.len() {
            let t = config.tags.get_unchecked(i);
            if t.len() > Self::MAX_TAG_LEN {
                return Err(Error::TagTooLong);
            }
        }

        if config.certifications.len() > Self::MAX_CERTIFICATIONS {
            return Err(Error::TooManyCertifications);
        }

        if config.media_hashes.len() > Self::MAX_MEDIA_HASHES {
            return Err(Error::TooManyMediaHashes);
        }

        Self::validate_custom_fields(&config.custom)
    }

    pub fn validate_deactivation_reason(reason: &String) -> Result<(), Error> {
        if reason.len() < 1 {
            return Err(Error::DeactivationReasonRequired);
        }
        Ok(())
    }

    // --- Custom fields / metadata validation ---
    pub fn validate_custom_fields(custom: &Map<Symbol, String>) -> Result<(), Error> {
        if custom.len() > Self::MAX_CUSTOM_FIELDS {
            return Err(Error::TooManyCustomFields);
        }

        let keys = custom.keys();
        for i in 0..keys.len() {
            let k = keys.get_unchecked(i);
            let v = custom.get_unchecked(k);
            if v.len() > Self::MAX_CUSTOM_VALUE_LEN {
                return Err(Error::CustomFieldValueTooLong);
            }
        }

        Ok(())
    }

    pub fn validate_metadata(metadata: &Map<Symbol, String>) -> Result<(), Error> {
        if metadata.len() > Self::MAX_METADATA_FIELDS {
            return Err(Error::TooManyCustomFields);
        }

        let keys = metadata.keys();
        for i in 0..keys.len() {
            let k = keys.get_unchecked(i);
            let v = metadata.get_unchecked(k);
            if v.len() > Self::MAX_CUSTOM_VALUE_LEN {
                return Err(Error::CustomFieldValueTooLong);
            }
        }

        Ok(())
    }

    // --- Event data validation ---
    pub fn validate_event_location(location: &String) -> Result<(), Error> {
        if location.len() > Self::MAX_LOCATION_LEN {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }

    pub fn validate_event_note(note: &String) -> Result<(), Error> {
        if note.len() > Self::MAX_NOTE_LEN {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }
}
