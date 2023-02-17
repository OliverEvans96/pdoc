use std::str::Chars;

use inquire::autocompletion::Replacement;

/// Iterate over strings in parallel, character-by-character.
/// Stop iterating when the first string is exhausted.
struct ParallelStringIterator<'a> {
    // strings: Vec<String>,
    // cur_ind: usize,
    iters: Vec<Chars<'a>>,
}

impl<'a> ParallelStringIterator<'a> {
    pub fn new(strings: &'a Vec<String>) -> Self {
        let iters = strings.into_iter().map(|s| s.chars()).collect();
        Self { iters }
    }
}

impl<'a> Iterator for ParallelStringIterator<'a> {
    type Item = Vec<char>;

    fn next(&mut self) -> Option<Self::Item> {
        // Collecting to an Option<Vec<_>> fails the whole operation if any one is missing,
        // thereby terminating iteration when the first string is exhausted.
        self.iters.iter_mut().map(|it| it.next()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ParallelStringIterator;

    #[test]
    fn test_par_str_iter() -> anyhow::Result<()> {
        let strings: Vec<String> = ["yellow", "red", "blue", "green"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect();

        let mut iter = ParallelStringIterator::new(&strings);

        assert_eq!(iter.next(), Some(['y', 'r', 'b', 'g'].to_vec()));
        assert_eq!(iter.next(), Some(['e', 'e', 'l', 'r'].to_vec()));
        assert_eq!(iter.next(), Some(['l', 'd', 'u', 'e'].to_vec()));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        Ok(())
    }
}

/// If all characters in the slice are the same,
/// return Some(char). Otherwise, return None.
fn chars_are_same(chars: &[char]) -> Option<char> {
    chars
        .split_first()
        .and_then(|(&first, rest)| rest.iter().all(|&c| c == first).then_some(first))
}

pub fn get_common_prefix(strings: Vec<String>) -> String {
    let par_iter = ParallelStringIterator::new(&strings);

    let mut common = String::new();

    for chars in par_iter {
        if let Some(next_char) = chars_are_same(&chars) {
            common.push(next_char);
        }
    }

    common
}

pub trait PrefixAutocomplete {
    fn get_options(&self) -> &[String];
    fn get_lowercase_options(&self) -> &[String];

    fn get_matches(&self, input: &str) -> anyhow::Result<Vec<String>> {
        let options = self.get_options();
        let lowercase_options = self.get_lowercase_options();

        let lowercase_input = input.to_lowercase();

        let matches = lowercase_options
            .iter()
            .enumerate()
            // Filter to matching names
            .filter(|(_i, name)| name.starts_with(&lowercase_input))
            // Get normal-case name with same index
            .map(|(i, _name)| options[i].clone())
            .collect();

        Ok(matches)
    }
}

#[derive(Clone)]
pub struct LocalAutocompleter<T: Clone + PrefixAutocomplete>(T);

impl<T> LocalAutocompleter<T>
where
    T: PrefixAutocomplete + Clone,
{
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> inquire::Autocomplete for LocalAutocompleter<T>
where
    T: PrefixAutocomplete + Clone,
{
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let matches = self.0.get_matches(input)?;
        Ok(matches)
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, inquire::CustomUserError> {
        if let Some(suggestion) = highlighted_suggestion {
            Ok(Replacement::Some(suggestion))
        } else {
            let matches = self.0.get_matches(input)?;
            let prefix = get_common_prefix(matches);
            Ok(Replacement::Some(prefix))
        }
    }
}
