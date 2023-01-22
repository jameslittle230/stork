pub(crate) mod parse_document;
pub(crate) mod read_contents;

use itertools::Itertools;

mod importance;
mod progress;

use importance::WordImportanceCalculator;
use rust_stemmers::Stemmer;

use crate::{
    build_config,
    build_output::{self, BuildStatistics, BuildSuccessValue},
    envelope::{self, Envelope},
    index_v4::{self, Document},
};

pub(crate) fn build_index(
    config: &build_config::Config,
    progress_fn: Option<&dyn Fn(build_output::ProgressReport)>,
) -> Result<build_output::BuildSuccessValue, build_output::errors::InternalBuildError> {
    let mut warnings: Vec<build_output::BuildWarning> = Vec::new();

    let progress_reporter = progress::ProgressReporter::new(config, progress_fn);

    let mut index = index_v4::Index::default_from_config(config);

    let documents = config
        .input
        .files
        .iter()
        .enumerate()
        .flat_map(|(document_id, file_config)| {
            progress_reporter.report(build_output::ProgressState::StartedDocument {
                index: document_id,
                title: file_config.title.clone(),
            });

            match read_contents::read_contents(config, document_id).and_then(|read_value| {
                parse_document::parse_document(config, document_id, &read_value)
            }) {
                Ok(document) => Some(document),
                Err(problem) => {
                    warnings.push((&problem).into());
                    None
                }
            }
        })
        .collect_vec();

    let mut importance_calc = WordImportanceCalculator::new();

    for (document_id, doc_parse_value) in documents.iter().enumerate() {
        index.documents.insert(
            document_id,
            Document::from_parse_value(document_id, doc_parse_value),
        );

        for title_word in &doc_parse_value.annotated_title_words {
            importance_calc.push(&title_word.word, document_id)
        }

        for word in &doc_parse_value.annotated_words {
            importance_calc.push(&word.word, document_id)
        }
    }

    for (document_id, doc_parse_value) in documents.iter().enumerate() {
        let stem_config = config.get_stem_config_for_file(document_id);
        let stemmer = match stem_config {
            crate::build_config::StemmingConfig::None => None,
            crate::build_config::StemmingConfig::Language(alg) => Some(Stemmer::create(alg)),
        };

        for title_word in &doc_parse_value.annotated_title_words {
            index.query_result_tree.insert_value_for_string(
                index_v4::QueryResult::TitleExcerpt(index_v4::TitleExcerpt {
                    document_id,
                    byte_offset: title_word.annotation.byte_offset,
                }),
                &title_word.word,
            );

            // TODO: Stem
        }

        for word in &doc_parse_value.annotated_words {
            index.query_result_tree.insert_value_for_string(
                index_v4::QueryResult::ContentsExcerpt(index_v4::ContentsExcerpt {
                    document_id,
                    byte_offset: word.annotation.byte_offset,
                    importance: importance_calc.get_value(&word.word, document_id),
                    url_suffix: word.annotation.url_suffix.clone(),
                }),
                &word.word,
            );

            // TODO: Stem
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
        warnings,
    })
}
