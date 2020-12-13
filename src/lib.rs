use hashbrown::HashMap;
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};
use unidecode::unidecode;

pub struct ThreeSetCompare {
    alphabet: Vec<char>,
    minimum_word_len: i32,
    delta_word_len_ignore: usize,
    min_word_similarity: f64,
    left_chars: Arc<Mutex<CharMap>>,
    right_chars: Arc<Mutex<CharMap>>,
}

type CharMap = HashMap<char, i32>;

enum Word {
    Left,
    Right,
}

impl ThreeSetCompare {
    pub fn new() -> ThreeSetCompare {
        let alphabet = (b'a'..=b'z')
            .chain(b'0'..=b'9')
            .map(|c| c as char)
            .collect::<Vec<_>>();

        let minimum_word_len = 2_i32;
        let delta_word_len_ignore = 3_usize;
        let min_word_similarity = 0.707_f64;
        let average_word_length = 20;

        ThreeSetCompare {
            alphabet,
            minimum_word_len,
            delta_word_len_ignore,
            min_word_similarity,

            left_chars: Arc::new(Mutex::new(CharMap::with_capacity(average_word_length))),
            right_chars: Arc::new(Mutex::new(CharMap::with_capacity(average_word_length))),
        }
    }

    #[inline(always)]
    fn count_chars(&self, data: &str, pos: Word) {
        let mut result = match pos {
            Word::Left => self.left_chars.lock().unwrap(),
            Word::Right => self.right_chars.lock().unwrap(),
        };

        result.clear();

        for letter in data.chars() {
            *result.entry(letter).or_insert(0) += 1;
        }
    }

    #[inline(always)]
    fn preprocess(&self, data: &str) -> Vec<String> {
        unidecode(data)
            .to_lowercase()
            .split_whitespace()
            .map(|word| word.to_string())
            .collect::<Vec<String>>()
    }

    fn logic(&self, first: &Vec<String>, second: &Vec<String>) -> f64 {
        let mut equality = 0;

        for first_word in first {
            for second_word in second {
                let first_len = first_word.chars().count() as i32;
                let second_len = second_word.chars().count() as i32;
                let delta_len = (first_len - second_len).abs() as usize;

                if first_len < self.minimum_word_len || second_len < self.minimum_word_len {
                    continue;
                }

                if first_word.find(second_word).is_some() || second_word.find(first_word).is_some()
                {
                    if delta_len <= self.delta_word_len_ignore {
                        equality += 1;
                    }
                } else {
                    self.count_chars(first_word, Word::Left);
                    self.count_chars(second_word, Word::Right);

                    let first_map = self.left_chars.lock().unwrap();
                    let second_map = self.right_chars.lock().unwrap();

                    let total_length = first_map
                        .iter()
                        .chain(second_map.iter())
                        .fold(0, |acc, (_, val)| acc + val);

                    let zero_count = 0;
                    let mut errors_sum = 0;

                    for alpha in &self.alphabet {
                        let count_first = first_map.get(&alpha).unwrap_or(&zero_count);
                        let count_second = second_map.get(&alpha).unwrap_or(&zero_count);

                        errors_sum += (count_first - count_second).abs();
                    }

                    let local_possibility = 1_f64 - (errors_sum as f64 / total_length as f64);
                    if local_possibility > self.min_word_similarity {
                        equality += 1;
                    }
                }
            }
        }

        let first_count_filtered = first
            .into_iter()
            .filter(|word| word.chars().count() >= self.minimum_word_len as usize)
            .count();

        let second_count_filtered = second
            .into_iter()
            .filter(|word| word.chars().count() >= self.minimum_word_len as usize)
            .count();

        let sum_count = (first_count_filtered + second_count_filtered) as f64 / 2_f64;
        f64::min(equality as f64 / sum_count as f64, 1.0)
    }

    /// Compare two strings for equality. Don't use this method with strings longer than 255 characters.
    /// You can use any language, the data is unidecoded before comparing.
    pub fn similarity(&self, first: &str, second: &str) -> f64 {
        let first_p = self.preprocess(first);
        let second_p = self.preprocess(second);

        return self.logic(&first_p, &second_p);
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    #[bench]
    fn bench_similarity(b: &mut Bencher) {
        let comparator = ThreeSetCompare::new();
        b.iter(|| {
            comparator.similarity(
                "Сравнеие двух строк с помощью инвариантной метрики",
                "Сравнеие двух строк с помощью метрики, инвариантной к перестановке слов",
            );
        });
    }

    use crate::ThreeSetCompare;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn differences() {
        let comparator = ThreeSetCompare::new();

        assert_approx_eq!(
            comparator.similarity(
                "Сравнение трех строк с помощью инвариантной метрики",
                "Сравнение двух строк с помощью инвариантной метрики"
            ),
            0.8333333_f64
        );
        assert_approx_eq!(
            comparator.similarity(
                "Сравнение трех строк   помощью инвариантной метрики",
                "Сравнение двух строк с помощью инвариантной метрики"
            ),
            0.8333333_f64
        );
        assert_approx_eq!(
            comparator.similarity(
                "Сравнеие двух строк с помощью инвариантной метрики",
                "Сравнеие двух строк с помощью метрики, инвариантной к перестановке слов"
            ),
            0.8571428_f64
        );
    }

    #[test]
    fn equal() {
        let comparator = ThreeSetCompare::new();

        assert_approx_eq!(
            comparator.similarity(
                "Сравнение двух строк с помощью инвариантной метрики",
                "Сравнение двух строк с помощью инвариантной метрики"
            ),
            1_f64
        );
        assert_approx_eq!(
            comparator.similarity(
                "Сравнение двух строк с помощью инвариантной метрики!",
                "Сравнение двух строк с помощью инвариантной метрики?"
            ),
            1_f64
        );
        assert_approx_eq!(
            comparator.similarity(
                "Сравнение двху строк с пмоощью инвариатнной метркии",
                "Сравнение двух строк с помощью инвариантной метрики"
            ),
            1_f64
        );
        assert_approx_eq!(
            comparator.similarity(
                "Сравнение строк двух с помощью метрики инвариантной",
                "Сравнение двух строк с помощью инвариантной метрики"
            ),
            1_f64
        );
    }

    #[test]
    fn not_equal() {
        let comparator = ThreeSetCompare::new();

        assert_approx_eq!(
            comparator.similarity("Первая строка", "Вторая фраза"),
            0.5_f64
        );

        assert_approx_eq!(comparator.similarity("АБВ", "ГДЕ"), 0_f64);
    }
}
