mod bucket_search_values;
mod build_output_result;
mod group_document_excerpts;
// mod line_item;
mod title_highlight_ranges;

use std::{time::Instant, vec};

use itertools::Itertools;

use super::{tree::TreeRetrievalValue, IndexDiskRepresentation};

use crate::{search_output::SearchOutput, search_query, search_value::SearchValue};

use crate::search_output::SearchResult;

pub(crate) fn get_search_values(
    index: &IndexDiskRepresentation,
    search_term: &search_query::SearchTerm,
) -> Vec<SearchValue> {
    let mut query_results = vec![];

    if !search_term.is_valid(&index.settings) {
        return vec![];
    }

    let values = match search_term {
        search_query::SearchTerm::InexactWord(word) => {
            let now = Instant::now();
            let v = index
                .query_tree
                .get_values_for_string(word, super::tree::GetValuesOption::Take(1000));
            println!(
                "{}us to get search values for {search_term:?}",
                now.elapsed().as_micros()
            );
            v
        }

        search_query::SearchTerm::ExactWord(word) => index
            .query_tree
            .get_values_for_string(word, super::tree::GetValuesOption::Exact),
    };

    if let Some(vec) = values {
        for TreeRetrievalValue {
            value,
            characters_remaining,
        } in vec
        {
            if let Some(result) = index.query_results.get(value) {
                query_results.push(SearchValue {
                    v4_value: Some(crate::search_value::V4SearchValue {
                        term: search_term.clone(),
                        result: result.clone(),
                        chars_remaining: characters_remaining,
                    }),
                });
            }
        }
    }

    query_results
}

pub(crate) fn resolve_search_values(
    index: &IndexDiskRepresentation,
    search_values: Vec<SearchValue>,
) -> SearchOutput {
    let v4_search_values = search_values
        .iter()
        .map(|value| value.v4_value.as_ref().unwrap())
        .collect_vec();

    let beginning = Instant::now();
    let sorted_excerpts = bucket_search_values::bucket_search_values_by_document(v4_search_values);
    println!(
        "{:>5}us to bucket search values",
        beginning.elapsed().as_micros()
    );

    let mut results: Vec<SearchResult> = sorted_excerpts
        .iter()
        .map(|(document_id, bucketed_excerpts)| {
            let group_start = Instant::now();
            let grouped_document_excerpts = group_document_excerpts::group_document_excerpts(
                bucketed_excerpts.document_contents_excerpts.clone(),
                50,
            );
            println!(
                "{:>5}us to group excerpts for document {document_id} ({} groups)",
                group_start.elapsed().as_micros(),
                grouped_document_excerpts.len()
            );

            let build_result_start = Instant::now();
            let result = build_output_result::build_search_result(
                index.documents.get(document_id).unwrap(),
                grouped_document_excerpts,
                bucketed_excerpts.title_excerpts.as_ref(),
                &index.settings.title_boost,
            );

            println!(
                "{:>5}us to build result for document {document_id}",
                build_result_start.elapsed().as_micros()
            );

            result

            // let mut excerpt_data_vec: Vec<line_item::SearchLineItem> = contents_excerpts
            //     .iter()
            //     .map(|ee| line_item::SearchLineItem {
            //         text: ee.output_excerpt.text.clone(),
            //         highlight_ranges: vec1::Vec1::try_from_vec(
            //             // TODO: Are there any cases where this conversion fails?
            //             ee.output_excerpt.highlight_ranges.clone(),
            //         )
            //         .unwrap(),
            //         content_offset: ee.document_contents_excerpt.contents_character_offset,
            //         score: ee.output_excerpt.score,
            //         fields: ee.output_excerpt.fields.clone(),
            //         internal_annotations: ee.output_excerpt.internal_annotations.clone(),
            //         url_suffix: ee.document_contents_excerpt.url_suffix.clone(),
            //     })
            //     .collect();

            // let mut excerpts: Vec<OutputExcerpt> =
            //     line_item::merge::merge_all_excerpts(&mut excerpt_data_vec)
            //         .iter()
            //         .map(|data| OutputExcerpt {
            //             text: data.text.clone(),
            //             highlight_ranges: data.highlight_ranges.clone().into(),
            //             score: data.score,
            //             internal_annotations: data.internal_annotations.clone(),
            //             fields: data.fields.clone(),
            //         })
            //         // .filter(|e| e.score > 20) // TODO: Don't filter if the query words are all stopwords
            //         .collect();

            // excerpts.sort_by_key(|e| e.score);
            // excerpts.reverse();
            // excerpts.truncate(10);

            // if let Some(last_resort) = last_resort_excerpts.get(&output_document.title) {
            //     if excerpts.is_empty() && !title_highlight_ranges.is_empty() {
            //         excerpts.push(last_resort.clone())
            //     }
            // }

            // // excerpts will be empty if the only excerpts were those for stopwords
            // if excerpts.is_empty() {
            //     return None;
            // }

            // (highest individual score / 4) * (sqrt(number of matching excerpts))
            // let excerpt_score_component: usize = ((excerpts.first().unwrap().score as f32 * 0.25)
            //     * ((contents_excerpts.len() as f32).powf(0.5)))
            //     as usize;

            // let title_score_component = (title_score_multiplier as f32
            //     * (title_highlight_ranges.len() as f32).powf(0.5))
            //     as usize;

            // let score = (excerpt_score_component * 10) + title_score_component;

            // Some(OutputResult {
            //     entry: output_document.clone(),
            //     excerpts,
            //     title_highlight_ranges: title_highlight_ranges.clone(),
            //     score,
            // })
        })
        .collect();

    // println!(
    //     "{}ms total to build search results",
    //     beginning.elapsed().as_millis()
    // );

    results.sort_by_key(|r| r.score);
    results.reverse();

    // println!(
    //     "{}ms total to build, sort, and reverse search results",
    //     beginning.elapsed().as_millis()
    // );

    let total_hit_count = results.len(); // TODO: truncate

    SearchOutput {
        results,
        total_hit_count,
        url_prefix: "".to_string(),
    }
}
