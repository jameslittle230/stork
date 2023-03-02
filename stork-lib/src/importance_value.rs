use minicbor::{Decode, Encode};
use std::hash::Hash;

#[derive(Debug, Clone, Encode, Decode)]
pub(crate) struct ImportanceValue(#[n(0)] pub(crate) f64);

impl Hash for ImportanceValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl PartialEq for ImportanceValue {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for ImportanceValue {
    fn assert_receiver_is_total_eq(&self) {}
}

impl From<f64> for ImportanceValue {
    fn from(other: f64) -> Self {
        ImportanceValue(other)
    }
}
