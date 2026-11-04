//! Error types for vyapaar-core.

use thiserror::Error;

/// Core domain errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    /// Invalid exchange identifier
    #[error("unknown exchange: '{0}' (expected NSE, BSE, NFO, MCX, or CDS)")]
    InvalidExchange(String),

    /// Unknown country code
    #[error("unknown country: '{0}' (expected IN, US, UK, or JP)")]
    InvalidCountry(String),

    /// Invalid decimal string for price
    #[error("failed to parse decimal price: '{0}'")]
    InvalidPrice(String),

    /// Negative price
    #[error("price cannot be negative: '{0}'")]
    NegativePrice(String),

    /// Invalid decimal string for quantity
    #[error("failed to parse decimal quantity: '{0}'")]
    InvalidQuantity(String),

    /// Negative quantity
    #[error("quantity cannot be negative: '{0}'")]
    NegativeQuantity(String),

    /// Unknown currency code
    #[error("unknown currency: '{0}' (expected INR, USD, EUR, or GBP)")]
    InvalidCurrency(String),

    /// Validation error for decisions or requests
    #[error("validation failure: {0}")]
    Validation(String),
}
