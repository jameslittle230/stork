use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use rust_stemmers::Stemmer;

mod importance;
mod progress;

use crate::{
    build_config::Config,
    build_output::{
        document_problem::AttributedDocumentProblem, errors::BuildErrorRepr, BuildStatistics,
        BuildSuccessValue, BuildWarning,
    },
    envelope::{self, Envelope},
    index_v4::{self, Document, Index},
};

pub(crate) use self::importance::ImportanceValue;
pub use self::progress::ProgressReporter;
use self::{importance::WordImportanceCalculator, parser::DocumentParseValue};

pub(crate) mod parser;
pub(crate) mod reader;

pub(crate) fn build_index(
    config: &Config,
    progress_reporter: impl ProgressReporter,
) -> Result<BuildSuccessValue, BuildErrorRepr> {
    let mut problems: Vec<AttributedDocumentProblem> = Vec::new();

    let mut handle_document_problem = |problem: &AttributedDocumentProblem| {
        problems.push(problem.clone());
        progress_reporter.send_warning(problem.to_string());
    };

    let mut index = Index::default_from_config(config);

    if config.input.files.is_empty() {
        return Err(BuildErrorRepr::NoFilesSpecified);
    }

    let mut documents: Vec<DocumentParseValue> = Vec::new();

    for (idx, file_config) in config.input.files.iter().enumerate() {
        progress_reporter.send_tick(idx, config.input.files.len(), file_config.title.clone());

        let parse_result = reader::read(config, idx)
            .and_then(|read_value| parser::parse(config, idx, &read_value));

        match parse_result {
            Ok(document) => documents.push(document),
            Err(problem) => {
                if config.output.break_on_first_error {
                    return Err(BuildErrorRepr::OneDocumentHadProblem(problem));
                } else {
                    handle_document_problem(&problem);
                }
            }
        }
    }

    if problems.len() == config.input.files.len() {
        progress_reporter.fail();
        return Err(BuildErrorRepr::AllDocumentsHadProblems(problems));
    }

    progress_reporter.succeed();

    let mut importance_calc = WordImportanceCalculator::new(config.input.files.len());
    let mut stem_map: HashMap<String, HashSet<String>> = HashMap::new(); // TODO: Does this become an OOM risk?

    for (document_id, doc_parse_value) in documents.iter().enumerate() {
        index.documents.insert(
            document_id,
            Document::from_parse_value(document_id, doc_parse_value),
        );

        let stem_config = config.get_stem_config_for_file(document_id);
        let stemmer = match stem_config {
            crate::build_config::StemmingConfig::None => None,
            crate::build_config::StemmingConfig::Language(alg) => Some(Stemmer::create(alg)),
        };

        for title_word in &doc_parse_value.annotated_title_words {
            let word = &title_word.word;
            importance_calc.push(word, document_id);

            if let Some(stemmer) = &stemmer {
                let stemmed_word = stemmer.stem(word).into_owned();
                stem_map
                    .entry(stemmed_word)
                    .and_modify(|v| {
                        v.insert(word.clone());
                    })
                    .or_insert_with(|| HashSet::from([word.clone()]));
            }
        }

        for word in &doc_parse_value.annotated_words {
            let word = &word.word;
            importance_calc.push(word, document_id);

            if let Some(stemmer) = &stemmer {
                let stemmed_word = stemmer.stem(word).into_owned();
                stem_map
                    .entry(stemmed_word)
                    .and_modify(|v| {
                        v.insert(word.clone());
                    })
                    .or_insert_with(|| HashSet::from([word.clone()]));
            }
        }
    }

    for (document_id, doc_parse_value) in documents.iter().enumerate() {
        let stem_config = config.get_stem_config_for_file(document_id);
        let stemmer = match stem_config {
            crate::build_config::StemmingConfig::None => None,
            crate::build_config::StemmingConfig::Language(alg) => Some(Stemmer::create(alg)),
        };

        for word in &doc_parse_value.annotated_words {
            println!("{}: {}", word.word, importance_calc.get_value(&word.word).0);
            index.query_result_tree.insert_value_for_string(
                index_v4::QueryResult::ContentsExcerpt(index_v4::ContentsExcerpt {
                    document_id,
                    byte_offset: word.annotation.byte_offset,
                    importance: importance_calc.get_value(&word.word),
                    url_suffix: word.annotation.url_suffix.clone(),
                }),
                &word.word,
            );

            // for string in get_stem_alternatives(&word.word, &stemmer, &stem_map) {
            //     index.query_result_tree.insert_value_for_string(
            //         index_v4::QueryResult::ContentsExcerpt(index_v4::ContentsExcerpt {
            //             document_id,
            //             byte_offset: word.annotation.byte_offset,
            //             importance: importance_calc.get_value(&word.word, document_id) / 3,
            //             url_suffix: word.annotation.url_suffix.clone(),
            //         }),
            //         &string,
            //     )
            // }
        }

        for word in &doc_parse_value.annotated_title_words {
            index.query_result_tree.insert_value_for_string(
                index_v4::QueryResult::TitleExcerpt(index_v4::TitleExcerpt {
                    document_id,
                    byte_offset: word.annotation.byte_offset,
                }),
                &word.word,
            );

            // for string in get_stem_alternatives(&word.word, &stemmer, &stem_map) {
            //     index.query_result_tree.insert_value_for_string(
            //         index_v4::QueryResult::TitleExcerpt(index_v4::TitleExcerpt {
            //             document_id,
            //             byte_offset: word.annotation.byte_offset,
            //         }),
            //         &string,
            //     )
            // }
        }
    }

    let (primary_data, sidecar_data) = index.to_disk_representation();
    Ok(BuildSuccessValue {
        primary_data: Envelope {
            prefix: envelope::Prefix::StorkV4Root,
            bytes: primary_data.to_bytes(),
        }
        .to_bytes(),

        sidecar_data: sidecar_data
            .iter()
            .map(|disk_rep| {
                Envelope {
                    prefix: envelope::Prefix::StorkV4Part,
                    bytes: disk_rep.to_bytes(),
                }
                .to_bytes()
            })
            .collect_vec(),

        statistics: BuildStatistics {
            entries_count: 0,
            tokens_count: 0,
            index_size_bytes: 0,
        },
        warnings: problems.iter().map(BuildWarning::from).collect_vec(),
    })
}

fn get_stem_alternatives(
    word: &str,
    stemmer: &Option<Stemmer>,
    stem_map: &HashMap<String, HashSet<String>>,
) -> Vec<String> {
    stemmer
        .as_ref()
        .and_then(|stemmer| {
            let key = stemmer.stem(word).into_owned();
            stem_map.get(&key).map(|hashset| {
                hashset
                    .iter()
                    .filter(|w| **w != word)
                    .cloned()
                    .collect_vec()
            })
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use pretty_assertions::assert_eq;
    use rust_stemmers::Stemmer;

    use crate::build::get_stem_alternatives;

    #[test]
    fn get_stem_alternatives_dedupes() {
        let mut computed = get_stem_alternatives(
            "liberties",
            &Some(Stemmer::create(rust_stemmers::Algorithm::English)),
            &HashMap::from([(
                "liberti".to_string(),
                HashSet::from([
                    "liberties".to_string(),
                    "liberty".to_string(),
                    "libertied".to_string(),
                ]),
            )]),
        );
        computed.sort();
        assert_eq!(
            computed,
            vec!["libertied".to_string(), "liberty".to_string()]
        );
    }
}
