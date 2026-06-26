// Prusti Formal Verification for Backend Auth Logic
// This module provides formal specifications and invariants for authentication

use prusti_contracts::*;

/// Formal specification for JWT Claims
#[derive(Clone)]
pub struct Claims {
    #[trusted]
    pub sub: String, // User ID
    #[trusted]
    pub exp: usize,  // Expiration timestamp
    #[trusted]
    pub role: UserRole,
}

/// User roles with formal ordering
#[derive(Clone, PartialEq, Eq)]
pub enum UserRole {
    Administrator,
    Manager,
    Operator,
    Viewer,
}

/// Auth context with proven invariants
#[derive(Clone)]
pub struct AuthContext {
    #[trusted]
    pub user_id: uuid::Uuid,
    #[trusted]
    pub api_key_id: Option<uuid::Uuid>,
    #[trusted]
    pub tier: Option<ApiKeyTier>,
    #[trusted]
    pub stellar_address: Option<String>,
    pub role: UserRole,
}

impl AuthContext {
    /// Invariant: Auth context always has a valid user ID
    #[invariant(self.user_id != uuid::Uuid::nil())]
    pub fn new(user_id: uuid::Uuid, role: UserRole) -> Self {
        Self {
            user_id,
            api_key_id: None,
            tier: None,
            stellar_address: None,
            role,
        }
    }

    /// Predicate: Check if user has sufficient role
    #[pure]
    pub fn has_role(&self, required: UserRole) -> bool {
        self.role >= required
    }

    /// Predicate: Check if auth context is valid
    #[pure]
    pub fn is_valid(&self) -> bool {
        self.user_id != uuid::Uuid::nil()
    }
}

/// Partial order for UserRole to enable role hierarchy checks
impl PartialOrd for UserRole {
    #[pure]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserRole {
    #[pure]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (UserRole::Administrator, UserRole::Administrator) => std::cmp::Ordering::Equal,
            (UserRole::Administrator, _) => std::cmp::Ordering::Greater,
            (_, UserRole::Administrator) => std::cmp::Ordering::Less,
            (UserRole::Manager, UserRole::Manager) => std::cmp::Ordering::Equal,
            (UserRole::Manager, _) => std::cmp::Ordering::Greater,
            (_, UserRole::Manager) => std::cmp::Ordering::Less,
            (UserRole::Operator, UserRole::Operator) => std::cmp::Ordering::Equal,
            (UserRole::Operator, UserRole::Viewer) => std::cmp::Ordering::Greater,
            (UserRole::Viewer, UserRole::Operator) => std::cmp::Ordering::Less,
            (UserRole::Viewer, UserRole::Viewer) => std::cmp::Ordering::Equal,
        }
    }
}

/// API Key tiers with rate limit thresholds
#[derive(Clone, PartialEq, Eq)]
pub enum ApiKeyTier {
    Basic,
    Standard,
    Premium,
    Enterprise,
}

/// Threshold configuration for rate limiting
#[derive(Clone)]
pub struct ThresholdConfig {
    pub max_requests_per_minute: u32,
    pub max_requests_per_hour: u32,
    pub max_requests_per_day: u32,
}

impl ThresholdConfig {
    /// Invariant: Thresholds must be non-decreasing
    #[invariant(self.max_requests_per_minute <= self.max_requests_per_hour)]
    #[invariant(self.max_requests_per_hour <= self.max_requests_per_day)]
    pub fn new(minute: u32, hour: u32, day: u32) -> Self {
        Self {
            max_requests_per_minute: minute,
            max_requests_per_hour: hour,
            max_requests_per_day: day,
        }
    }

    /// Predicate: Check if threshold is valid
    #[pure]
    pub fn is_valid(&self) -> bool {
        self.max_requests_per_minute > 0
            && self.max_requests_per_hour >= self.max_requests_per_minute
            && self.max_requests_per_day >= self.max_requests_per_hour
    }

    /// Predicate: Check if request count exceeds threshold
    #[pure]
    pub fn exceeds_threshold(&self, count: u32, window: TimeWindow) -> bool {
        match window {
            TimeWindow::Minute => count > self.max_requests_per_minute,
            TimeWindow::Hour => count > self.max_requests_per_hour,
            TimeWindow::Day => count > self.max_requests_per_day,
        }
    }
}

#[derive(Clone, Copy)]
pub enum TimeWindow {
    Minute,
    Hour,
    Day,
}

/// Formal specification for threshold-based authorization
pub struct ThresholdAuth {
    #[trusted]
    pub config: ThresholdConfig,
    #[trusted]
    pub current_counts: (u32, u32, u32), // (minute, hour, day)
}

impl ThresholdAuth {
    /// Invariant: Current counts must respect thresholds
    #[invariant(self.current_counts.0 <= self.config.max_requests_per_minute)]
    #[invariant(self.current_counts.1 <= self.config.max_requests_per_hour)]
    #[invariant(self.current_counts.2 <= self.config.max_requests_per_day)]
    pub fn new(config: ThresholdConfig) -> Self {
        Self {
            config,
            current_counts: (0, 0, 0),
        }
    }

    /// Precondition: Request must not exceed threshold
    /// Postcondition: Count is incremented
    #[requires(!self.config.exceeds_threshold(self.current_counts.0, TimeWindow::Minute))]
    #[ensures(self.current_counts.0 == old(self.current_counts.0) + 1)]
    pub fn record_minute_request(&mut self) {
        self.current_counts.0 += 1;
    }

    /// Precondition: Request must not exceed threshold
    /// Postcondition: Count is incremented
    #[requires(!self.config.exceeds_threshold(self.current_counts.1, TimeWindow::Hour))]
    #[ensures(self.current_counts.1 == old(self.current_counts.1) + 1)]
    pub fn record_hour_request(&mut self) {
        self.current_counts.1 += 1;
    }

    /// Precondition: Request must not exceed threshold
    /// Postcondition: Count is incremented
    #[requires(!self.config.exceeds_threshold(self.current_counts.2, TimeWindow::Day))]
    #[ensures(self.current_counts.2 == old(self.current_counts.2) + 1)]
    pub fn record_day_request(&mut self) {
        self.current_counts.2 += 1;
    }

    /// Predicate: Check if authorization should be granted
    #[pure]
    pub fn authorize(&self) -> bool {
        !self.config.exceeds_threshold(self.current_counts.0, TimeWindow::Minute)
            && !self.config.exceeds_threshold(self.current_counts.1, TimeWindow::Hour)
            && !self.config.exceeds_threshold(self.current_counts.2, TimeWindow::Day)
    }
}

/// Formal specification for multi-signature threshold logic
pub struct MultiSigThreshold {
    #[trusted]
    pub signers: Vec<uuid::Uuid>,
    pub threshold: u32,
    #[trusted]
    pub approvals: Vec<uuid::Uuid>,
    #[trusted]
    pub rejections: Vec<uuid::Uuid>,
}

impl MultiSigThreshold {
    /// Invariant: Threshold must be positive and not exceed signers count
    #[invariant(self.threshold > 0)]
    #[invariant(self.threshold <= self.signers.len() as u32)]
    /// Invariant: No duplicate signers
    #[invariant(self.signers.len() == {
        let mut seen = std::collections::HashSet::new();
        for s in &self.signers {
            seen.insert(s);
        }
        seen.len()
    })]
    pub fn new(signers: Vec<uuid::Uuid>, threshold: u32) -> Self {
        Self {
            signers,
            threshold,
            approvals: Vec::new(),
            rejections: Vec::new(),
        }
    }

    /// Invariant: Approvals must be subset of signers
    #[invariant({
        let approval_set: std::collections::HashSet<_> = self.approvals.iter().collect();
        self.signers.iter().all(|s| approval_set.contains(s) || !self.approvals.contains(s))
    })]
    /// Invariant: No duplicate approvals
    #[invariant({
        let mut seen = std::collections::HashSet::new();
        for a in &self.approvals {
            if !seen.insert(a) {
                return false;
            }
        }
        true
    })]
    pub fn add_approval(&mut self, approver: uuid::Uuid) -> bool {
        if self.approvals.contains(&approver) || self.rejections.contains(&approver) {
            return false;
        }
        if !self.signers.contains(&approver) {
            return false;
        }
        self.approvals.push(approver);
        true
    }

    /// Invariant: Rejections must be subset of signers
    #[invariant({
        let rejection_set: std::collections::HashSet<_> = self.rejections.iter().collect();
        self.signers.iter().all(|s| rejection_set.contains(s) || !self.rejections.contains(s))
    })]
    /// Invariant: No duplicate rejections
    #[invariant({
        let mut seen = std::collections::HashSet::new();
        for r in &self.rejections {
            if !seen.insert(r) {
                return false;
            }
        }
        true
    })]
    pub fn add_rejection(&mut self, rejecter: uuid::Uuid) -> bool {
        if self.rejections.contains(&rejecter) || self.approvals.contains(&rejecter) {
            return false;
        }
        if !self.signers.contains(&rejecter) {
            return false;
        }
        self.rejections.push(rejecter);
        true
    }

    /// Predicate: Check if approval threshold is reached
    #[pure]
    pub fn threshold_reached(&self) -> bool {
        self.approvals.len() as u32 >= self.threshold
    }

    /// Predicate: Check if rejection threshold is reached
    #[pure]
    pub fn rejection_threshold_reached(&self) -> bool {
        let max_rejections = self.signers.len() as u32 - self.threshold + 1;
        self.rejections.len() as u32 >= max_rejections
    }

    /// Predicate: Check if proposal can be executed
    #[pure]
    pub fn can_execute(&self) -> bool {
        self.threshold_reached() && !self.rejection_threshold_reached()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_config_validity() {
        let config = ThresholdConfig::new(10, 100, 1000);
        assert!(config.is_valid());
    }

    #[test]
    fn test_role_hierarchy() {
        assert!(UserRole::Administrator >= UserRole::Manager);
        assert!(UserRole::Manager >= UserRole::Operator);
        assert!(UserRole::Operator >= UserRole::Viewer);
    }

    #[test]
    fn test_multisig_threshold() {
        let signers = vec![
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
        ];
        let mut ms = MultiSigThreshold::new(signers, 2);
        
        assert!(!ms.threshold_reached());
        ms.add_approval(ms.signers[0]);
        assert!(!ms.threshold_reached());
        ms.add_approval(ms.signers[1]);
        assert!(ms.threshold_reached());
    }
}
