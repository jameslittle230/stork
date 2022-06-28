use rust_stemmers::Algorithm as StemAlgorithm;

/**
 * The first layer of document processing. Holds the document's metadata and its
 * contents after those contents have gone through the word separation algorithm.
 */
pub(super) struct WordSegmentedDocument {
    pub(super) annotated_words: Vec<AnnotatedWord>,
    pub(super) contents: String, // The word separation algorithm is lossy - you're not guaranteed to be able to get the original contents back from the word list.
    pub(super) stem_algorithm: Option<StemAlgorithm>,
    pub(super) title: String,
    pub(super) url: String,
}

#[derive(Debug, Clone)]
pub(crate) struct AnnotatedWord {
    pub(crate) word: String,
    pub(crate) annotation: WordAnnotation,
}

impl AnnotatedWord {
    pub(crate) fn new(word: String, character_offset: usize, url_suffix: Option<String>) -> Self {
        Self {
            word,
            annotation: WordAnnotation {
                character_offset,
                url_suffix,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WordAnnotation {
    pub(crate) character_offset: usize,
    pub(crate) url_suffix: Option<String>,
}
