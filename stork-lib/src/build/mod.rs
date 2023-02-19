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
        warnings: problems.iter().map(BuildWarning::from).collect_vec(),
    })
}
