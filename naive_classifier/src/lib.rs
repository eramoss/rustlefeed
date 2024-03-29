use feed_rs::model::Entry;
use regex::Regex;
use rusqlite::{Connection, Row};
use std::collections::{HashMap, HashSet};

pub struct NaiveBayesClassifier {
    pub alpha: f64,
    pub tokens: HashSet<String>,
    pub token_liked_counts: HashMap<String, i32>,
    pub token_disliked_counts: HashMap<String, i32>,
    pub disliked_entries_count: i32,
    pub liked_entries_count: i32,
    pub is_prepared: bool,
}

type PossiblyLiked = bool;

#[derive(Debug, Clone, PartialEq)]
pub struct EntryContent {
    all_content: String,
    liked: PossiblyLiked,
}

impl EntryContent {
    pub fn from_row(row: &Row) -> EntryContent {
        let title: String = row.get("title").unwrap_or_default();
        let summary: String = row.get("summary").unwrap_or_default();
        let content: String = row.get("content").unwrap_or_default();
        let authors: String = row.get("authors").unwrap_or_default();
        let categories: String = row.get("categories").unwrap_or_default();
        let link: String = row.get("link").unwrap_or_default();

        let all_content = format!(
            "{} {} {} {} {} {}",
            title, summary, content, authors, categories, link
        );

        let liked = row.get("is_liked").unwrap();
        EntryContent { all_content, liked }
    }
}

impl NaiveBayesClassifier {
    pub fn new(db_path: &str) -> Result<NaiveBayesClassifier, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS already_seen (
                id TEXT PRIMARY KEY,
                title TEXT,
                authors TEXT,
                content TEXT,
                links TEXT,
                summary TEXT,
                categories TEXT,
                language TEXT,
                is_liked INTEGER
            )",
            [],
        )?;
        let mut stmt = conn.prepare("SELECT * FROM already_seen")?;
        let entry_contents = stmt
            .query_map([], |row| Ok(EntryContent::from_row(row)))?
            .collect::<Vec<Result<EntryContent, rusqlite::Error>>>();

        let entry_contents = entry_contents
            .into_iter()
            .map(|entry_content| entry_content.unwrap())
            .collect::<Vec<EntryContent>>();

        let mut classifier = NaiveBayesClassifier::new_classifier(1.0);
        classifier.is_prepared = entry_contents.len() > 100;
        if classifier.is_prepared {
            classifier.train(entry_contents);
        }
        Ok(classifier)
    }

    pub(crate) fn new_classifier(alpha: f64) -> NaiveBayesClassifier {
        NaiveBayesClassifier {
            alpha,
            tokens: HashSet::new(),
            token_liked_counts: HashMap::new(),
            token_disliked_counts: HashMap::new(),
            disliked_entries_count: 0,
            liked_entries_count: 0,
            is_prepared: false,
        }
    }

    pub fn train(&mut self, data: Vec<EntryContent>) {
        for entry in data.iter() {
            self.increment_entry_classifications_count(entry);
            for token in Self::tokenize(&entry.all_content) {
                self.tokens.insert(token.to_string());
                self.increment_token_count(token, entry.liked)
            }
        }
    }

    pub fn classify(&self, entry: Entry) -> f64 {
        if !self.is_prepared {
            return 1.;
        }
        let link = match entry.links.get(0) {
            Some(link) => link.href.to_lowercase(),
            None => String::new(),
        };
        let text = format!(
            "{} {} {} {} {} {}",
            entry.title.unwrap_or_default().content,
            entry.summary.unwrap_or_default().content,
            entry.content.unwrap_or_default().body.unwrap_or_default(),
            entry
                .authors
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            entry
                .categories
                .iter()
                .map(|c| c.term.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            link,
        );
        let lower_case_text = text.to_lowercase();
        let message_tokens = Self::tokenize(&lower_case_text);
        let (prob_if_dislike, prob_if_liked) = self.probabilities_of_message(message_tokens);
        return prob_if_liked / (prob_if_liked + prob_if_dislike);
    }
    fn probabilities_of_message(&self, message_tokens: HashSet<&str>) -> (f64, f64) {
        let mut log_prob_if_dislike = 0.;
        let mut log_prob_if_like = 0.;
        let epsilon = 1e-9;

        for token in self.tokens.iter() {
            let (prob_if_disliked, prob_if_like) = self.probabilites_of_token(&token);

            let prob_if_disliked = prob_if_disliked.max(epsilon).min(1. - epsilon);
            let prob_if_like = prob_if_like.max(epsilon).min(1. - epsilon);

            if message_tokens.contains(token.as_str()) {
                log_prob_if_dislike += prob_if_disliked.ln();
                log_prob_if_like += prob_if_like.ln();
            } else {
                log_prob_if_dislike += (1. - prob_if_disliked).ln();
                log_prob_if_like += (1. - prob_if_like).ln();
            }
        }

        let prob_if_dislike = log_prob_if_dislike.exp();
        let prob_if_like = log_prob_if_like.exp();

        return (prob_if_dislike, prob_if_like);
    }

    fn probabilites_of_token(&self, token: &str) -> (f64, f64) {
        let prob_of_token_disliked = (self.token_disliked_counts[token] as f64 + self.alpha)
            / (self.liked_entries_count as f64 + 2. * self.alpha);

        let prob_of_token_liked = (self.token_liked_counts[token] as f64 + self.alpha)
            / (self.liked_entries_count as f64 + 2. * self.alpha);

        return (prob_of_token_disliked, prob_of_token_liked);
    }

    fn increment_entry_classifications_count(&mut self, entry: &EntryContent) {
        if entry.liked {
            self.disliked_entries_count += 1;
        } else {
            self.liked_entries_count += 1;
        }
    }

    fn increment_token_count(&mut self, token: &str, liked: bool) {
        if !self.token_disliked_counts.contains_key(token) {
            self.token_disliked_counts.insert(token.to_string(), 0);
        }

        if !self.token_liked_counts.contains_key(token) {
            self.token_liked_counts.insert(token.to_string(), 0);
        }

        if liked {
            self.increment_liked_count(token);
        } else {
            self.increment_disliked_count(token);
        }
    }

    fn increment_disliked_count(&mut self, token: &str) {
        *self.token_disliked_counts.get_mut(token).unwrap() += 1;
    }

    fn increment_liked_count(&mut self, token: &str) {
        *self.token_liked_counts.get_mut(token).unwrap() += 1;
    }

    pub(crate) fn tokenize(lower_case_text: &str) -> HashSet<&str> {
        Regex::new(r"[a-z0-9']+")
            .unwrap()
            .find_iter(lower_case_text)
            .map(|mat| mat.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use feed_rs::model::Content;

    use super::*;

    #[test]
    fn naive_bayes() {
        let train_messages = [
            EntryContent {
                all_content: "Free Bitcoin viagra XXX christmas deals 😻😻😻".to_string(),
                liked: true,
            },
            EntryContent {
                all_content: "My dear Granddaughter, please explain Bitcoin over Christmas dinner"
                    .to_string(),
                liked: false,
            },
            EntryContent {
                all_content: "Here in my garage...".to_string(),
                liked: true,
            },
        ];

        let alpha = 1.;
        let num_spam_messages = 2.;
        let num_ham_messages = 1.;

        let mut model = NaiveBayesClassifier::new_classifier(alpha);
        model.train(train_messages.to_vec());

        let mut expected_tokens: HashSet<String> = HashSet::new();
        for message in train_messages.iter() {
            for token in NaiveBayesClassifier::tokenize(&message.all_content.to_lowercase()) {
                expected_tokens.insert(token.to_string());
            }
        }

        let input_text = "Bitcoin crypto academy Christmas deals";

        let probs_if_spam = [
            1. - (1. + alpha) / (num_spam_messages + 2. * alpha), // "Free"  (not present)
            (1. + alpha) / (num_spam_messages + 2. * alpha),      // "Bitcoin"  (present)
            1. - (1. + alpha) / (num_spam_messages + 2. * alpha), // "viagra"  (not present)
            1. - (1. + alpha) / (num_spam_messages + 2. * alpha), // "XXX"  (not present)
            (1. + alpha) / (num_spam_messages + 2. * alpha),      // "christmas"  (present)
            (1. + alpha) / (num_spam_messages + 2. * alpha),      // "deals"  (present)
            1. - (1. + alpha) / (num_spam_messages + 2. * alpha), // "my"  (not present)
            1. - (0. + alpha) / (num_spam_messages + 2. * alpha), // "dear"  (not present)
            1. - (0. + alpha) / (num_spam_messages + 2. * alpha), // "granddaughter"  (not present)
            1. - (0. + alpha) / (num_spam_messages + 2. * alpha), // "please"  (not present)
            1. - (0. + alpha) / (num_spam_messages + 2. * alpha), // "explain"  (not present)
            1. - (0. + alpha) / (num_spam_messages + 2. * alpha), // "over"  (not present)
            1. - (0. + alpha) / (num_spam_messages + 2. * alpha), // "dinner"  (not present)
            1. - (1. + alpha) / (num_spam_messages + 2. * alpha), // "here"  (not present)
            1. - (1. + alpha) / (num_spam_messages + 2. * alpha), // "in"  (not present)
            1. - (1. + alpha) / (num_spam_messages + 2. * alpha), // "garage"  (not present)
        ];

        let probs_if_ham = [
            1. - (0. + alpha) / (num_ham_messages + 2. * alpha), // "Free"  (not present)
            (1. + alpha) / (num_ham_messages + 2. * alpha),      // "Bitcoin"  (present)
            1. - (0. + alpha) / (num_ham_messages + 2. * alpha), // "viagra"  (not present)
            1. - (0. + alpha) / (num_ham_messages + 2. * alpha), // "XXX"  (not present)
            (1. + alpha) / (num_ham_messages + 2. * alpha),      // "christmas"  (present)
            (0. + alpha) / (num_ham_messages + 2. * alpha),      // "deals"  (present)
            1. - (1. + alpha) / (num_ham_messages + 2. * alpha), // "my"  (not present)
            1. - (1. + alpha) / (num_ham_messages + 2. * alpha), // "dear"  (not present)
            1. - (1. + alpha) / (num_ham_messages + 2. * alpha), // "granddaughter"  (not present)
            1. - (1. + alpha) / (num_ham_messages + 2. * alpha), // "please"  (not present)
            1. - (1. + alpha) / (num_ham_messages + 2. * alpha), // "explain"  (not present)
            1. - (1. + alpha) / (num_ham_messages + 2. * alpha), // "over"  (not present)
            1. - (1. + alpha) / (num_ham_messages + 2. * alpha), // "dinner"  (not present)
            1. - (0. + alpha) / (num_ham_messages + 2. * alpha), // "here"  (not present)
            1. - (0. + alpha) / (num_ham_messages + 2. * alpha), // "in"  (not present)
            1. - (0. + alpha) / (num_ham_messages + 2. * alpha), // "garage"  (not present)
        ];

        let p_if_spam_log: f64 = probs_if_spam.iter().map(|p| p.ln()).sum();
        let p_if_spam = p_if_spam_log.exp();

        let p_if_ham_log: f64 = probs_if_ham.iter().map(|p| p.ln()).sum();
        let p_if_ham = p_if_ham_log.exp();
        let mut entry = Entry::default();
        entry.content = Some(Content {
            body: Some(input_text.to_string()),
            ..Default::default()
        });
        // P(message | spam) / (P(messge | spam) + P(message | ham)) rounds to 0.97

        assert!((model.classify(entry) - p_if_spam / (p_if_spam + p_if_ham)).abs() < 0.034);
    }
}
