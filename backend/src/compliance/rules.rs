use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceType {
    GDPR,
    FDA21CFR11,
    FSMA,
    ConflictMinerals,
    OrganicCertification,
    SOC2,
    ISO27001,
}

impl ComplianceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComplianceType::GDPR => "gdpr",
            ComplianceType::FDA21CFR11 => "fda_21_cfr_11",
            ComplianceType::FSMA => "fsma",
            ComplianceType::ConflictMinerals => "conflict_minerals",
            ComplianceType::OrganicCertification => "organic_certification",
            ComplianceType::SOC2 => "soc2",
            ComplianceType::ISO27001 => "iso27001",
        }
    }

    pub fn retention_days(&self) -> u32 {
        match self {
            ComplianceType::GDPR => 2555, // 7 years
            ComplianceType::FDA21CFR11 => 3650, // 10 years
            ComplianceType::FSMA => 9125, // 25 years
            ComplianceType::ConflictMinerals => 1825, // 5 years
            ComplianceType::OrganicCertification => 1095, // 3 years
            ComplianceType::SOC2 => 1095, // 3 years
            ComplianceType::ISO27001 => 1095, // 3 years
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    pub rule_id: String,
    pub compliance_type: ComplianceType,
    pub description: String,
    pub validation_logic: String,
    pub required_fields: Vec<String>,
    pub is_active: bool,
}

impl ComplianceRule {
    pub fn gdpr_data_residency() -> Self {
        ComplianceRule {
            rule_id: "gdpr_residency".to_string(),
            compliance_type: ComplianceType::GDPR,
            description: "Ensure personal data is stored in EU data centers".to_string(),
            validation_logic: "data_location == 'EU'".to_string(),
            required_fields: vec!["data_location".to_string(), "consent".to_string()],
            is_active: true,
        }
    }

    pub fn fda_electronic_signature() -> Self {
        ComplianceRule {
            rule_id: "fda_esig".to_string(),
            compliance_type: ComplianceType::FDA21CFR11,
            description: "Require electronic signatures for pharmaceutical records".to_string(),
            validation_logic: "has_digital_signature && signature_timestamp".to_string(),
            required_fields: vec!["digital_signature".to_string(), "signer_id".to_string()],
            is_active: true,
        }
    }

    pub fn fsma_traceability() -> Self {
        ComplianceRule {
            rule_id: "fsma_trace".to_string(),
            compliance_type: ComplianceType::FSMA,
            description: "Maintain complete traceability for food products".to_string(),
            validation_logic: "has_complete_chain_of_custody".to_string(),
            required_fields: vec![
                "origin".to_string(),
                "processing_steps".to_string(),
                "distribution".to_string(),
            ],
            is_active: true,
        }
    }

    pub fn conflict_minerals_due_diligence() -> Self {
        ComplianceRule {
            rule_id: "conflict_minerals".to_string(),
            compliance_type: ComplianceType::ConflictMinerals,
            description: "Verify conflict-free mineral sourcing".to_string(),
            validation_logic: "has_due_diligence_report && verified_by_auditor".to_string(),
            required_fields: vec!["mineral_source".to_string(), "audit_report".to_string()],
            is_active: true,
        }
    }

    pub fn organic_certification() -> Self {
        ComplianceRule {
            rule_id: "organic_cert".to_string(),
            compliance_type: ComplianceType::OrganicCertification,
            description: "Verify organic certification status".to_string(),
            validation_logic: "has_valid_organic_cert && cert_not_expired".to_string(),
            required_fields: vec!["certification_body".to_string(), "cert_expiry".to_string()],
            is_active: true,
        }
    }
}
