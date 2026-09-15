//! Governance repository — CRUD for `governance_contracts` and `policy_rules`.

use rusqlite::{Connection, Row, params};

use agileplus_domain::{
    domain::governance::{
        GovernanceContract, GovernanceRule, PolicyDefinition, PolicyDomain, PolicyRule,
    },
    error::DomainError,
};

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn policy_domain_str(d: PolicyDomain) -> &'static str {
    d.as_str()
}

fn policy_domain_from_str(s: &str) -> Result<PolicyDomain, DomainError> {
    match s {
        "security" => Ok(PolicyDomain::Security),
        "quality" => Ok(PolicyDomain::Quality),
        "compliance" => Ok(PolicyDomain::Compliance),
        "performance" => Ok(PolicyDomain::Performance),
        "custom" => Ok(PolicyDomain::Custom),
        _ => Err(DomainError::Storage(format!("invalid policy domain: {s}"))),
    }
}

// -- Governance Contracts --

pub fn create_governance_contract(
    conn: &Connection,
    contract: &GovernanceContract,
) -> Result<i64, DomainError> {
    let rules_json =
        serde_json::to_string(&contract.rules).map_err(|e| DomainError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO governance_contracts (feature_id, version, rules, bound_at)
         VALUES (?1,?2,?3,?4)",
        params![
            contract.feature_id,
            contract.version,
            rules_json,
            contract.bound_at.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;

    Ok(conn.last_insert_rowid())
}

fn row_to_contract_parts(row: &Row<'_>) -> rusqlite::Result<(i64, i64, i32, String, String)> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

fn parse_contract(
    parts: (i64, i64, i32, String, String),
) -> Result<GovernanceContract, DomainError> {
    let (id, feature_id, version, rules_json, bound_at_s) = parts;

    let rules: Vec<GovernanceRule> =
        serde_json::from_str(&rules_json).map_err(|e| DomainError::Storage(e.to_string()))?;
    let bound_at = bound_at_s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| DomainError::Storage(e.to_string()))?;

    Ok(GovernanceContract {
        id,
        feature_id,
        version,
        rules,
        bound_at,
    })
}

pub fn get_governance_contract(
    conn: &Connection,
    feature_id: i64,
    version: i32,
) -> Result<Option<GovernanceContract>, DomainError> {
    conn.query_row(
        "SELECT id,feature_id,version,rules,bound_at
         FROM governance_contracts WHERE feature_id=?1 AND version=?2",
        params![feature_id, version],
        row_to_contract_parts,
    )
    .optional()
    .map_err(map_err)?
    .map(parse_contract)
    .transpose()
}

pub fn get_latest_governance_contract(
    conn: &Connection,
    feature_id: i64,
) -> Result<Option<GovernanceContract>, DomainError> {
    conn.query_row(
        "SELECT id,feature_id,version,rules,bound_at
         FROM governance_contracts WHERE feature_id=?1 ORDER BY version DESC LIMIT 1",
        params![feature_id],
        row_to_contract_parts,
    )
    .optional()
    .map_err(map_err)?
    .map(parse_contract)
    .transpose()
}

// -- Policy Rules --

pub fn create_policy_rule(conn: &Connection, rule: &PolicyRule) -> Result<i64, DomainError> {
    let rule_json =
        serde_json::to_string(&rule.rule).map_err(|e| DomainError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO policy_rules (domain, rule, active, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5)",
        params![
            policy_domain_str(rule.domain),
            rule_json,
            rule.active as i32,
            rule.created_at.to_rfc3339(),
            rule.updated_at.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;

    Ok(conn.last_insert_rowid())
}

pub fn list_active_policies(conn: &Connection) -> Result<Vec<PolicyRule>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id,domain,rule,active,created_at,updated_at
             FROM policy_rules WHERE active = 1",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?
        .into_iter()
        .map(
            |(id, domain_s, rule_json, active, created_at_s, updated_at_s)| {
                let domain = policy_domain_from_str(&domain_s)?;
                let rule: PolicyDefinition = serde_json::from_str(&rule_json)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                let created_at = created_at_s
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                let updated_at = updated_at_s
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(PolicyRule {
                    id,
                    domain,
                    rule,
                    active: active != 0,
                    created_at,
                    updated_at,
                })
            },
        )
        .collect()
}

/// Extension trait to add `.optional()` on rusqlite query results.
trait OptionalExt<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}

impl<T> OptionalExt<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;
    use agileplus_domain::domain::governance::{
        GovernanceContract, GovernanceRule, PolicyCheck, PolicyRule,
    };

    fn seed_feature(conn: &Connection, id: i64) {
        conn.execute(
            "INSERT OR IGNORE INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'created', X'00', 'main', datetime('now'), datetime('now'))",
            params![id, format!("feat-{id}"), format!("Feature {id}")],
        )
        .unwrap();
    }

    fn sample_contract(feature_id: i64, version: i32) -> GovernanceContract {
        GovernanceContract {
            id: 0,
            feature_id,
            version,
            rules: vec![GovernanceRule {
                transition: "deploy".to_string(),
                required_evidence: vec!["test".to_string()],
                policy_refs: vec![],
            }],
            bound_at: chrono::Utc::now(),
        }
    }

    fn sample_policy(domain: PolicyDomain) -> PolicyRule {
        PolicyRule {
            id: 0,
            domain,
            rule: PolicyDefinition {
                description: "test".to_string(),
                check: PolicyCheck::Automated,
            },
            active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_create_and_get_contract() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 42);
        let id = create_governance_contract(&conn, &sample_contract(42, 1)).unwrap();
        assert!(id > 0);
        let c = get_governance_contract(&conn, 42, 1).unwrap().unwrap();
        assert_eq!(c.feature_id, 42);
        assert_eq!(c.version, 1);
    }

    #[test]
    fn test_get_contract_nonexistent_returns_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 999);
        assert!(get_governance_contract(&conn, 999, 1).unwrap().is_none());
    }

    #[test]
    fn test_contract_versioning() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 42);
        create_governance_contract(&conn, &sample_contract(42, 1)).unwrap();
        create_governance_contract(&conn, &sample_contract(42, 2)).unwrap();
        let v1 = get_governance_contract(&conn, 42, 1).unwrap().unwrap();
        let v2 = get_governance_contract(&conn, 42, 2).unwrap().unwrap();
        assert_eq!(v1.version, 1);
        assert_eq!(v2.version, 2);
    }

    #[test]
    fn test_get_latest_contract() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 42);
        create_governance_contract(&conn, &sample_contract(42, 1)).unwrap();
        create_governance_contract(&conn, &sample_contract(42, 3)).unwrap();
        let latest = get_latest_governance_contract(&conn, 42).unwrap().unwrap();
        assert_eq!(latest.version, 3);
    }

    #[test]
    fn test_get_latest_contract_nonexistent() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 999);
        assert!(get_latest_governance_contract(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn test_create_and_list_policies() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_policy_rule(&conn, &sample_policy(PolicyDomain::Security)).unwrap();
        create_policy_rule(&conn, &sample_policy(PolicyDomain::Quality)).unwrap();
        let active = list_active_policies(&conn).unwrap();
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_inactive_policies_not_listed() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_policy_rule(&conn, &sample_policy(PolicyDomain::Security)).unwrap();
        let mut inactive = sample_policy(PolicyDomain::Quality);
        inactive.active = false;
        create_policy_rule(&conn, &inactive).unwrap();
        let active = list_active_policies(&conn).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].domain, PolicyDomain::Security);
    }

    #[test]
    fn test_policy_domain_roundtrips() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        for domain in [
            PolicyDomain::Security,
            PolicyDomain::Quality,
            PolicyDomain::Compliance,
            PolicyDomain::Performance,
            PolicyDomain::Custom,
        ] {
            create_policy_rule(&conn, &sample_policy(domain)).unwrap();
        }
        let active = list_active_policies(&conn).unwrap();
        assert_eq!(active.len(), 5);
    }
}
