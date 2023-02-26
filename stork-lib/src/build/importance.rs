use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
    ops::{Add, Div},
};

use minicbor::{Decode, Encode};

#[derive(Debug)]
pub(crate) struct WordImportanceCalculator {
    //documents joined into a single HashMap
    pub term_freqs: BTreeMap<String, usize>,

    // key: the ter
    pub doc_freqs: BTreeMap<String, HashSet<usize>>,

    //total count of words inserted
    word_count: usize,

    total_document_count: usize,
}

impl WordImportanceCalculator {
    pub(crate) fn new(total_document_count: usize) -> Self {
        Self {
            term_freqs: BTreeMap::new(),
            doc_freqs: BTreeMap::new(),
            word_count: 0,
            total_document_count,
        }
    }

    pub(crate) fn push(&mut self, word: &str, document_id: usize) {
        self.term_freqs
            .entry(word.to_string())
            .and_modify(|e| *e += 1)
            .or_insert(1);

        self.doc_freqs
            .entry(word.to_string())
            .and_modify(|e| {
                e.insert(document_id);
            })
            .or_insert_with(|| HashSet::from([document_id]));

        self.word_count += 1;
    }

    /// How often a word appears in a document, divided by how many words
    /// there are in the corpus.
    fn tf(&self, word: &str) -> f64 {
        self.term_freqs
            .get(word)
            .map(|freq| *freq as f64 / self.word_count as f64)
            .expect("Tried getting a term from the importance calculator that wasn't inserted")
    }

    /// Log of (total number of documents / number of documents with the term in it)
    fn idf(&self, word: &str) -> f64 {
        let doc_freq = self
            .doc_freqs
            .get(word)
            .map(|hs| hs.len() as f64)
            .unwrap_or_default();

        let ratio = self.total_document_count as f64 / doc_freq;

        (ratio + 10.0).ln()
    }

    pub(crate) fn get_value(&self, word: &str) -> ImportanceValue {
        ImportanceValue(1.0 / (self.tf(word) * self.idf(word)))
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
