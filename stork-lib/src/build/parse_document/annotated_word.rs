#[derive(Debug, Clone)]
pub(crate) struct AnnotatedWord {
    pub(crate) word: String,
    pub(crate) annotation: WordAnnotation,
}

impl AnnotatedWord {
    pub(crate) fn new(word: String, byte_offset: usize, url_suffix: Option<String>) -> Self {
        Self {
            word,
            annotation: WordAnnotation {
                byte_offset,
                url_suffix,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WordAnnotation {
    pub(crate) byte_offset: usize,
    pub(crate) url_suffix: Option<String>,
}
