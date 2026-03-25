use soroban_sdk::{testutils::Address as _, Address, Env, Map, String, Vec};

use crate::{
    AuthorizationContract, AuthorizationContractClient, ProductConfig, ProductRegistryContract,
    ProductRegistryContractClient, ProductTransferContract, ProductTransferContractClient,
};

// Benchmarks here are deterministic workload benchmarks intended to track
// relative gas/footprint trends between changes. They are not wall-clock tests.

fn setup(env: &Env) -> (ProductRegistryContractClient, ProductTransferContractClient) {
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let registry_id = env.register_contract(None, ProductRegistryContract);
    let transfer_id = env.register_contract(None, ProductTransferContract);

    let auth_client = AuthorizationContractClient::new(env, &auth_id);
    let registry_client = ProductRegistryContractClient::new(env, &registry_id);
    let transfer_client = ProductTransferContractClient::new(env, &transfer_id);

    auth_client.configure_initializer(&registry_id);
    registry_client.configure_auth_contract(&auth_id);
    transfer_client.pt_init(&registry_id, &auth_id);

    (registry_client, transfer_client)
}

fn register_bulk(
    env: &Env,
    registry: &ProductRegistryContractClient,
    owner: &Address,
    offset: u32,
    count: u32,
) -> Vec<String> {
    let mut ids = Vec::new(env);
    for i in 0..count {
        let idx = (offset + i) % 10_000;
        let d0 = ((idx / 1000) % 10) as u8;
        let d1 = ((idx / 100) % 10) as u8;
        let d2 = ((idx / 10) % 10) as u8;
        let d3 = (idx % 10) as u8;
        let id = String::from_bytes(
            env,
            &[
                b'B',
                b'E',
                b'N',
                b'C',
                b'H',
                b'-',
                b'0' + d0,
                b'0' + d1,
                b'0' + d2,
                b'0' + d3,
            ],
        );
        registry.register_product(
            owner,
            &ProductConfig {
                id: id.clone(),
                name: String::from_str(env, "Bench Product"),
                description: String::from_str(env, "bench"),
                origin_location: String::from_str(env, "Nairobi"),
                category: String::from_str(env, "Food"),
                tags: Vec::new(env),
                certifications: Vec::new(env),
                media_hashes: Vec::new(env),
                custom: Map::new(env),
            },
        );
        ids.push_back(id);
    }
    ids
}

#[test]
fn benchmark_registration_scales_to_500_products() {
    let env = Env::default();
    let (registry, _transfer) = setup(&env);
    let owner = Address::generate(&env);

    let sizes = [50u32, 100u32, 250u32, 500u32];
    let mut base = 0u32;
    for size in sizes {
        let ids = register_bulk(&env, &registry, &owner, base, size);
        assert_eq!(ids.len(), size);
        base += size;
    }

    let stats = registry.get_stats();
    assert_eq!(stats.total_products, 900);
    assert_eq!(stats.active_products, 900);
}

#[test]
#[ignore = "benchmark workload, run manually in performance pipeline"]
fn benchmark_batch_transfer_chunks_of_100() {
    let env = Env::default();
    let (registry, transfer) = setup(&env);
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);

    let ids = register_bulk(&env, &registry, &owner, 0, 300);
    let mut moved = 0u32;

    let mut start = 0u32;
    while start < ids.len() {
        let end = (start + 100).min(ids.len());
        let chunk = ids.slice(start..end);
        moved += transfer.batch_transfer_products(&owner, &chunk, &new_owner);
        start = end;
    }

    assert_eq!(moved, 300);
    assert_eq!(
        registry
            .get_product(&String::from_str(&env, "BENCH-0000"))
            .owner,
        new_owner
    );
    assert_eq!(
        registry
            .get_product(&String::from_str(&env, "BENCH-0299"))
            .owner,
        new_owner
    );
}
