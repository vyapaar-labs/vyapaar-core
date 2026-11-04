//! Canonical strongly-typed aggregate & entity identifiers.
//!
//! Promoted into the Shared Kernel (Tier 0) from the Domain Heart's
//! in-memory repository layer, so that every bounded context —
//! [`vyapaar-ledger`], [`vyapaar-risk`], [`vyapaar-market`], and
//! [`vyapaar-exec`] — shares identical, strongly-typed IDs instead of
//! redefining private stringly-typed duplicates (split-brain ubiquitous
//! language fix).
//!
//! [`vyapaar-ledger`]: ../../vyapaar-ledger/index.html
//! [`vyapaar-risk`]: ../../vyapaar-risk/index.html
//! [`vyapaar-market`]: ../../vyapaar-market/index.html
//! [`vyapaar-exec`]: ../../vyapaar-exec/index.html

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Generates a canonical identifier value object: string-backed, ordered,
/// self-validating by construction (any owned string is a valid ID).
macro_rules! canonical_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
        )]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// Wrap a raw identifier string.
            #[must_use]
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            /// Borrow the raw identifier.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }
    };
}

canonical_id! {
    /// Strongly-typed Client Order Identifier.
    ///
    /// The client-side correlation ID minted at order creation and carried
    /// through the risk gate, execution gateway, OMS, and ledger. Distinct
    /// from the broker-assigned venue order id ([`crate::instrument::VenueOrderId`]).
    ClientOrderId
}

canonical_id! {
    /// Strongly-typed Algorithm Identifier (SEBI algo-trading registration).
    ///
    /// Every order request, decision, and audit event carries an `AlgoId` so
    /// the exchange's algo audit trail is preserved end to end.
    AlgoId
}

canonical_id! {
    /// Strongly-typed Trading Account Identifier.
    ///
    /// Identifies the trading account (live, paper, or backtest) an order,
    /// position, or ledger transaction belongs to.
    AccountId
}

canonical_id! {
    /// Strongly-typed Ledger Transaction Identifier.
    ///
    /// Identifies one double-entry posting (or a balanced group of postings)
    /// in the general ledger journal.
    TransactionId
}

impl ClientOrderId {
    /// Wrap a minted [`uuid::Uuid`] into its canonical string form.
    #[must_use]
    pub fn from_uuid(u: uuid::Uuid) -> Self {
        Self(u.to_string())
    }
}

impl From<uuid::Uuid> for ClientOrderId {
    fn from(u: uuid::Uuid) -> Self {
        ClientOrderId::from_uuid(u)
    }
}
