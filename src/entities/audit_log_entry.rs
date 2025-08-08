use std::net::IpAddr;

use sqlx::{PgExecutor, postgres::PgQueryResult, types::ipnetwork::IpNetwork};
use uuid::Uuid;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "audit_log_action", rename_all = "snake_case")]
pub enum AuditLogAction {
    SecretRead,
    SecretWrite,
}

/// An individual entry in the Audit Log. Note that the struct is completely
/// empty at the moment, as we do not query any audit log entries in this app.
#[derive(Debug)]
pub struct AuditLogEntry {}

impl AuditLogEntry {
    /// Stores an action in the audit log. It will always assume that the action
    /// happened at the current timestamp. The IP addressed passed into it will
    /// be canonicalized.
    pub async fn log_action<'e>(
        db: impl PgExecutor<'e>,
        client_addr: IpAddr,
        action: AuditLogAction,
        token: Uuid,
        secret: Uuid,
    ) -> Result<PgQueryResult, sqlx::Error> {
        let client_addr = client_addr.to_canonical();
        let ip_net = match client_addr {
            IpAddr::V4(_) => IpNetwork::new(client_addr, 32),
            IpAddr::V6(_) => IpNetwork::new(client_addr, 128),
        }
        .expect("IP address provided here should always be valid");

        sqlx::query!(
            "insert into audit_log (client_addr, action, token, secret) values ($1, $2, $3, $4)",
            ip_net,
            action as AuditLogAction,
            token,
            secret
        )
        .execute(db)
        .await
    }
}
