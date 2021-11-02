use super::super::structs::AnnotatedWord;
use crate::common::InternalWordAnnotation;

pub(super) trait AnnotatedWordable {
    fn make_annotated_words(&self) -> Vec<AnnotatedWord>;
    fn make_annotated_words_with_annotations<F>(&self, closure: F) -> Vec<AnnotatedWord>
    where
        F: Fn(&str, &mut Vec<InternalWordAnnotation>);
}

impl AnnotatedWordable for str {
    fn make_annotated_words(&self) -> Vec<AnnotatedWord> {
        self.make_annotated_words_with_annotations(|_, _| {})
    }

    fn make_annotated_words_with_annotations<F: Fn(&str, &mut Vec<InternalWordAnnotation>)>(
        &self,
        closure: F,
    ) -> Vec<AnnotatedWord> {
        self.split(|c: char| c.is_ascii_whitespace() || c == '-')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|w| {
                let mut internal_annotations: Vec<InternalWordAnnotation> = Vec::new();
                closure(w, &mut internal_annotations);
                AnnotatedWord {
                    word: w.to_string(),
                    internal_annotations,
                    ..AnnotatedWord::default()
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::common::InternalWordAnnotation;

    use super::AnnotatedWordable;

    #[test]
    fn annotated_words_split_on_hyphens() {
        let expected: usize = 3;
        let computed = "Hastings-on-hudson".make_annotated_words().len();
        assert_eq!(expected, computed);
    }

    #[test]
    fn annotated_words_split_on_whitespace() {
        let expected: usize = 3;
        let computed = "Hastings on hudson".make_annotated_words().len();
        assert_eq!(expected, computed);
    }

    #[test]
    fn annotated_words_split_on_multiple_whitespace() {
        let expected: usize = 3;
        let computed = "Hastings         on  \n \t hudson"
            .make_annotated_words()
            .len();
        assert_eq!(expected, computed);
    }

    #[test]
    fn annotated_words_can_correctly_annotate() {
        let computed = "Hastings         on  \n \t hudson".make_annotated_words_with_annotations(
            |word, vec| vec.push(InternalWordAnnotation::UrlSuffix(word.to_string())),
        );

        assert_eq!(3, computed.len());
        assert_eq!(1, computed[0].internal_annotations.len());
        assert_eq!(
            InternalWordAnnotation::UrlSuffix("Hastings".to_string()),
            computed[0].internal_annotations[0]
        );
        assert_eq!(
            InternalWordAnnotation::UrlSuffix("hudson".to_string()),
            computed[2].internal_annotations[0]
        );
    }
}
