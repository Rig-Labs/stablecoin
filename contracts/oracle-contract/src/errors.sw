library;

// Oracle errors
pub enum OracleError {
    // Debug is not enabled.
    DebugNotEnabled: (),
    // Negative value.
    NegativeValue: (),
    // Multiplication would exceed u64 maximum.
    MultiplicationWouldExceedU64Maximum: (),
    // Price value exceeds u64 maximum.
    PriceValueExceedsU64Maximum: (),
}