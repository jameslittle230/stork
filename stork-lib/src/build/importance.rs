use std::collections::BTreeMap;

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
            .or_insert(BTreeMap::from([(word.to_string(), 1)]));
    }

    pub(crate) fn get_value(&self, term: &str, document_id: usize) -> f64 {
        let docs = self.documents.values();
        let v =
            1.0 / MyTfIdfStrategy::tfidf(
                term.to_string(),
                self.documents.get(&document_id).unwrap(),
                docs,
            ) * 100.0;

        println!("{document_id} {term}\t{v}");
        v
    }
}
