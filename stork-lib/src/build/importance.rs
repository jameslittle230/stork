use std::{
    collections::BTreeMap,
    hash::Hash,
    ops::{Add, Div},
};

use minicbor::{Decode, Encode};
use tfidf::{idf::InverseFrequencySmoothIdf, tf::LogNormalizationTf, ProcessedDocument, TfIdf};

pub(crate) struct WordImportanceCalculator {
    documents: BTreeMap<usize, BTreeMap<String, usize>>,
}

#[derive(Copy, Clone)]
struct MyTfIdfStrategy;

impl<T> TfIdf<T> for MyTfIdfStrategy
where
    T: ProcessedDocument,
{
    type Tf = LogNormalizationTf;
    type Idf = InverseFrequencySmoothIdf;
}

impl WordImportanceCalculator {
    pub(crate) fn new() -> Self {
        Self {
            documents: BTreeMap::default(),
        }
    }

    pub(crate) fn push(&mut self, word: &str, document_id: usize) {
        self.documents
            .entry(document_id)
            .and_modify(|map| {
                map.entry(word.to_string())
                    .and_modify(|count| {
                        *count += 1;
                    })
                    .or_insert(1);
            })
            .or_insert_with(|| BTreeMap::from([(word.to_string(), 1)]));
    }

    pub(crate) fn get_value(&self, term: &str, document_id: usize) -> ImportanceValue {
        let docs = self.documents.values();
        let v =
            1.0 / MyTfIdfStrategy::tfidf(
                term.to_string(),
                self.documents.get(&document_id).unwrap(),
                docs,
            ) * 100.0;

        ImportanceValue(v)
    }
}

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

impl Div for ImportanceValue {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        ImportanceValue(self.0 / rhs.0)
    }
}

impl Div<usize> for ImportanceValue {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        ImportanceValue(self.0 / (rhs as f64))
    }
}

impl Add<f64> for ImportanceValue {
    type Output = f64;

    fn add(self, rhs: f64) -> Self::Output {
        self.0 + rhs
    }
}
