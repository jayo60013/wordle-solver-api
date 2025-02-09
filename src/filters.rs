pub fn filter_by_grey_letters(word: &str, grey_letters: &[char]) -> bool {
    // If any grey letter is contained in the word -> word is invalid
    !grey_letters.iter().any(|letter| word.contains(*letter))
}

pub fn filter_by_yellow_letters(word: &str, yellow_letters: &[(char, usize)]) -> bool {
    // If any yellow letter isn't in the word -> word is invalid
    let all_yellow_letters_in_word = yellow_letters
        .iter()
        .all(|(letter, _)| word.contains(*letter));

    if !all_yellow_letters_in_word {
        return false;
    }

    // If any word has a yellow letter at that position -> word is invalid
    yellow_letters
        .iter()
        .all(|(letter, position)| word.chars().nth(*position) != Some(*letter))
}

pub fn filter_by_green_letters(word: &str, green_letters: &[(char, usize)]) -> bool {
    // If any word has a green letter not at that position -> word is invalid
    green_letters
        .iter()
        .all(|(letter, position)| word.chars().nth(*position) == Some(*letter))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_grey_letters_false() {
        // Given
        let word: String = "apple".to_string();
        let letters = vec!['a'];

        // When
        let actual = filter_by_grey_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }

    #[test]
    fn test_filter_by_grey_letters_true() {
        // Given
        let word: String = "reign".to_string();
        let letters = vec!['a', 'b', 'c', 'd'];

        // When
        let actual = filter_by_grey_letters(&word, &letters);

        // Then
        assert_eq!(actual, true);
    }

    #[test]
    fn test_filter_by_yellow_letters_does_not_contain_letter() {
        // Given
        let word: String = "apple".to_string();
        let letters = vec![('z', 0), ('p', 4)];

        // When
        let actual = filter_by_yellow_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }

    #[test]
    fn test_filter_by_yellow_letters_contains_letter_at_position() {
        // Given
        let word: String = "apple".to_string();
        let letters = vec![('a', 0)];

        // When
        let actual = filter_by_yellow_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }

    #[test]
    fn test_filter_by_yellow_letters_valid() {
        // Given
        let word: String = "share".to_string();
        let letters = vec![('r', 0), ('s', 3)];

        // When
        let actual = filter_by_yellow_letters(&word, &letters);

        // Then
        assert_eq!(actual, true);
    }

    #[test]
    fn test_filter_by_green_letters_does_not_contain_letter() {
        // Given
        let word: String = "share".to_string();
        let letters = vec![('t', 0)];

        // When
        let actual = filter_by_green_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }

    #[test]
    fn test_filter_by_green_letters_contains_letter_at_wrong_position() {
        // Given
        let word: String = "share".to_string();
        let letters = vec![('r', 0)];

        // When
        let actual = filter_by_green_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }

    #[test]
    fn test_filter_by_green_letters_contains_letter_at_correct_position() {
        // Given
        let word: String = "share".to_string();
        let letters = vec![('r', 3)];

        // When
        let actual = filter_by_green_letters(&word, &letters);

        // Then
        assert_eq!(actual, true);
    }
}
