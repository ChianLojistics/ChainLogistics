/// Role-Based Access Control (RBAC) for ChainLogistics smart contracts (#224).
///
/// Implements a five-tier permission model for supply chain operations:
///
/// | Role       | Permissions                                      |
/// |------------|--------------------------------------------------|
/// | Admin      | Full system control; manage all roles            |
/// | Supplier   | Register products; update own shipments          |
/// | Carrier    | Update transportation status and location data   |
/// | Inspector  | Add quality-check and certification events       |
/// | Customer   | Read-only access to own transactions             |
///
/// All role assignments and revocations emit structured events so off-chain
/// audit indexers can maintain an authoritative permission log.
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

use crate::error::Error;
use crate::storage;

// ── Role enum ─────────────────────────────────────────────────────────────────

/// Supply chain participant roles.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Supplier,
    Carrier,
    Inspector,
    Customer,
}

// ── Storage keys ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
enum RbacKey {
    /// role assignment: (address, role) -> bool
    HasRole(Address, Role),
}

// ── Event topics ──────────────────────────────────────────────────────────────

const TOPIC_ROLE_GRANTED: Symbol = symbol_short!("RoleGrant");
const TOPIC_ROLE_REVOKED: Symbol = symbol_short!("RoleRevok");
const TOPIC_ACCESS_DENIED: Symbol = symbol_short!("AccsDeny");

// ── Storage helpers ───────────────────────────────────────────────────────────

/// Returns `true` if `address` holds `role`.
pub fn has_role(env: &Env, address: &Address, role: &Role) -> bool {
    env.storage()
        .persistent()
        .get::<RbacKey, bool>(&RbacKey::HasRole(address.clone(), role.clone()))
        .unwrap_or(false)
}

fn set_role(env: &Env, address: &Address, role: &Role, value: bool) {
    env.storage()
        .persistent()
        .set(&RbacKey::HasRole(address.clone(), role.clone()), &value);
}

// ── Public RBAC functions ─────────────────────────────────────────────────────

/// Assign `role` to `grantee`.
/// Only the admin may grant roles. The calling contract function is
/// responsible for calling `caller.require_auth()` before invoking this.
///
/// Emits a `RoleGrant` event with `(grantee, role_name)` data.
pub fn grant_role(env: &Env, caller: &Address, grantee: &Address, role: Role) -> Result<(), Error> {
    require_role_or_fail(env, caller, &Role::Admin)?;

    set_role(env, grantee, &role, true);

    env.events().publish(
        (TOPIC_ROLE_GRANTED, role.clone()),
        (grantee.clone(),),
    );

    Ok(())
}

/// Revoke `role` from `grantee`.
/// Only the admin may revoke roles. The calling contract function is
/// responsible for calling `caller.require_auth()` before invoking this.
///
/// Emits a `RoleRevok` event with `(grantee, role_name)` data.
pub fn revoke_role(env: &Env, caller: &Address, grantee: &Address, role: Role) -> Result<(), Error> {
    require_role_or_fail(env, caller, &Role::Admin)?;

    set_role(env, grantee, &role, false);

    env.events().publish(
        (TOPIC_ROLE_REVOKED, role.clone()),
        (grantee.clone(),),
    );

    Ok(())
}

/// Check whether `address` holds `role`, logging a security event and
/// returning `Unauthorized` if not.
///
/// Emits an `AccsDeny` event when access is denied so off-chain monitors
/// can detect probing or attempted privilege escalation.
pub fn require_role_or_fail(env: &Env, address: &Address, role: &Role) -> Result<(), Error> {
    // Admin is implicitly granted if the caller is the contract admin.
    if matches!(role, Role::Admin) {
        if let Some(admin) = storage::get_admin(env) {
            if &admin == address {
                return Ok(());
            }
        }
    }

    if has_role(env, address, role) {
        return Ok(());
    }

    env.events().publish(
        (TOPIC_ACCESS_DENIED, role.clone()),
        (address.clone(),),
    );

    Err(Error::Unauthorized)
}

/// Multi-signature guard: require that `caller` holds at least one of the
/// listed roles. Returns on the first matching role.
pub fn require_any_role(env: &Env, caller: &Address, roles: &[Role]) -> Result<Role, Error> {
    for role in roles {
        if require_role_or_fail(env, caller, role).is_ok() {
            return Ok(role.clone());
        }
    }
    env.events().publish(
        (TOPIC_ACCESS_DENIED, symbol_short!("AnyRole")),
        (caller.clone(),),
    );
    Err(Error::Unauthorized)
}

// ── Convenience modifier wrappers ─────────────────────────────────────────────

/// Modifier: caller must be Supplier or Admin.
pub fn require_supplier(env: &Env, caller: &Address) -> Result<(), Error> {
    require_any_role(env, caller, &[Role::Supplier, Role::Admin]).map(|_| ())
}

/// Modifier: caller must be Carrier or Admin.
pub fn require_carrier(env: &Env, caller: &Address) -> Result<(), Error> {
    require_any_role(env, caller, &[Role::Carrier, Role::Admin]).map(|_| ())
}

/// Modifier: caller must be Inspector or Admin.
pub fn require_inspector(env: &Env, caller: &Address) -> Result<(), Error> {
    require_any_role(env, caller, &[Role::Inspector, Role::Admin]).map(|_| ())
}

/// Modifier: caller must hold at least one active supply chain role
/// (Supplier, Carrier, or Inspector). Customers are read-only and Admin
/// is handled separately.
pub fn require_active_participant(env: &Env, caller: &Address) -> Result<(), Error> {
    require_any_role(
        env,
        caller,
        &[Role::Supplier, Role::Carrier, Role::Inspector, Role::Admin],
    )
    .map(|_| ())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChainLogisticsContract, ChainLogisticsContractClient, AuthorizationContract};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    /// Set up a registered ChainLogisticsContract so storage ops run inside a
    /// valid contract context. Returns the contract id, admin, and a spare user.
    fn setup() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let auth_id = env.register(AuthorizationContract, ());
        let contract_id = env.register(ChainLogisticsContract, ());
        let client = ChainLogisticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin, &auth_id);

        let user = Address::generate(&env);
        (env, contract_id, admin, user)
    }

    #[test]
    fn test_grant_and_check_role() {
        let (env, cid, admin, user) = setup();
        env.as_contract(&cid, || {
            grant_role(&env, &admin, &user, Role::Supplier).unwrap();
            assert!(has_role(&env, &user, &Role::Supplier));
            assert!(!has_role(&env, &user, &Role::Carrier));
        });
    }

    #[test]
    fn test_revoke_role() {
        let (env, cid, admin, user) = setup();
        env.as_contract(&cid, || {
            grant_role(&env, &admin, &user, Role::Carrier).unwrap();
            revoke_role(&env, &admin, &user, Role::Carrier).unwrap();
            assert!(!has_role(&env, &user, &Role::Carrier));
        });
    }

    #[test]
    fn test_unauthorized_grant_rejected() {
        let (env, cid, _admin, user) = setup();
        let impostor = Address::generate(&env);
        env.as_contract(&cid, || {
            let result = grant_role(&env, &impostor, &user, Role::Supplier);
            assert_eq!(result, Err(Error::Unauthorized));
        });
    }

    #[test]
    fn test_require_any_role_passes_for_first_match() {
        let (env, cid, admin, user) = setup();
        env.as_contract(&cid, || {
            grant_role(&env, &admin, &user, Role::Inspector).unwrap();
            let matched =
                require_any_role(&env, &user, &[Role::Carrier, Role::Inspector]).unwrap();
            assert_eq!(matched, Role::Inspector);
        });
    }

    #[test]
    fn test_require_any_role_fails_when_none_match() {
        let (env, cid, _admin, user) = setup();
        env.as_contract(&cid, || {
            let result = require_any_role(&env, &user, &[Role::Supplier, Role::Carrier]);
            assert_eq!(result, Err(Error::Unauthorized));
        });
    }

    #[test]
    fn test_admin_implicitly_passes_any_require() {
        let (env, cid, admin, _user) = setup();
        env.as_contract(&cid, || {
            assert!(require_supplier(&env, &admin).is_ok());
            assert!(require_carrier(&env, &admin).is_ok());
            assert!(require_inspector(&env, &admin).is_ok());
        });
    }

    #[test]
    fn test_customer_role_not_active_participant() {
        let (env, cid, admin, user) = setup();
        env.as_contract(&cid, || {
            grant_role(&env, &admin, &user, Role::Customer).unwrap();
            let result = require_active_participant(&env, &user);
            assert_eq!(result, Err(Error::Unauthorized));
        });
    }
}
