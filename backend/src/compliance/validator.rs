use crate::compliance::{ComplianceRule, ComplianceType};
use serde_json::{json, Value};

pub struct ComplianceValidator;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_compliant: bool,
    pub compliance_type: ComplianceType,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}

impl ComplianceValidator {
    pub fn validate(rule: &ComplianceRule, data: &Value) -> ValidationResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // Check required fields
        for field in &rule.required_fields {
            if data.get(field).is_none() {
                violations.push(format!("Missing required field: {}", field));
            }
        }

        // Type-specific validations
        match rule.compliance_type {
            ComplianceType::GDPR => {
                Self::validate_gdpr(data, &mut violations, &mut warnings);
            }
            ComplianceType::FDA21CFR11 => {
                Self::validate_fda(data, &mut violations, &mut warnings);
            }
            ComplianceType::FSMA => {
                Self::validate_fsma(data, &mut violations, &mut warnings);
            }
            ComplianceType::ConflictMinerals => {
                Self::validate_conflict_minerals(data, &mut violations, &mut warnings);
            }
            ComplianceType::OrganicCertification => {
                Self::validate_organic(data, &mut violations, &mut warnings);
            }
            _ => {}
        }

        ValidationResult {
            is_compliant: violations.is_empty(),
            compliance_type: rule.compliance_type,
            violations,
            warnings,
        }
    }

    fn validate_gdpr(data: &Value, violations: &mut Vec<String>, warnings: &mut Vec<String>) {
        if let Some(location) = data.get("data_location") {
            if location.as_str() != Some("EU") {
                violations.push("GDPR: Data must be stored in EU".to_string());
            }
        }

        if data.get("consent").is_none() {
            violations.push("GDPR: User consent required".to_string());
        }

        if data.get("right_to_be_forgotten_enabled").is_none() {
            warnings.push("GDPR: Right to be forgotten not configured".to_string());
        }
    }

    fn validate_fda(data: &Value, violations: &mut Vec<String>, _warnings: &mut Vec<String>) {
        if data.get("digital_signature").is_none() {
            violations.push("FDA 21 CFR Part 11: Digital signature required".to_string());
        }

        if data.get("signer_id").is_none() {
            violations.push("FDA 21 CFR Part 11: Signer identification required".to_string());
        }

        if data.get("timestamp").is_none() {
            violations.push("FDA 21 CFR Part 11: Timestamp required".to_string());
        }
    }

    fn validate_fsma(data: &Value, violations: &mut Vec<String>, _warnings: &mut Vec<String>) {
        if data.get("origin").is_none() {
            violations.push("FSMA: Product origin required".to_string());
        }

        if data.get("processing_steps").is_none() {
            violations.push("FSMA: Processing steps required".to_string());
        }

        if data.get("distribution").is_none() {
            violations.push("FSMA: Distribution information required".to_string());
        }
    }

    fn validate_conflict_minerals(data: &Value, violations: &mut Vec<String>, warnings: &mut Vec<String>) {
        if data.get("mineral_source").is_none() {
            violations.push("Conflict Minerals: Source information required".to_string());
        }

        if data.get("audit_report").is_none() {
            violations.push("Conflict Minerals: Audit report required".to_string());
        }

        if data.get("verified_by_auditor").as_bool() != Some(true) {
            warnings.push("Conflict Minerals: Awaiting auditor verification".to_string());
        }
    }

    fn validate_organic(data: &Value, violations: &mut Vec<String>, warnings: &mut Vec<String>) {
        if data.get("certification_body").is_none() {
            violations.push("Organic: Certification body required".to_string());
        }

        if let Some(expiry) = data.get("cert_expiry") {
            if let Some(expiry_str) = expiry.as_str() {
                if let Ok(expiry_date) = chrono::DateTime::parse_from_rfc3339(expiry_str) {
                    if expiry_date < chrono::Utc::now() {
                        violations.push("Organic: Certification expired".to_string());
                    } else if expiry_date.timestamp() - chrono::Utc::now().timestamp() < 7776000 {
                        // 90 days
                        warnings.push("Organic: Certification expiring soon".to_string());
                    }
                }
            }
        }
    }

    pub fn generate_compliance_report(validations: &[ValidationResult]) -> Value {
        let total = validations.len();
        let compliant = validations.iter().filter(|v| v.is_compliant).count();
        let non_compliant = total - compliant;

        json!({
            "total_checks": total,
            "compliant": compliant,
            "non_compliant": non_compliant,
            "compliance_rate": if total > 0 { (compliant as f64 / total as f64) * 100.0 } else { 0.0 },
            "details": validations.iter().map(|v| {
                json!({
                    "type": v.compliance_type.as_str(),
                    "is_compliant": v.is_compliant,
                    "violations": v.violations,
                    "warnings": v.warnings,
                })
            }).collect::<Vec<_>>()
        })
    }
}
