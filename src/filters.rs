//! Askama filters

use crate::me::PaymentMethod;
/// Display logic for payment method
pub fn payment_to_latex(method: &PaymentMethod) -> askama::Result<String> {
    Ok(method.to_latex())
}
