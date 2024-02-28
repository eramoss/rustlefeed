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
}

type PossiblyLiked = bool;

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
        let mut stmt = conn.prepare("SELECT * FROM already_seen")?;
        let entry_contents = stmt
            .query_map([], |row| Ok(EntryContent::from_row(row)))?
            .collect::<Vec<Result<EntryContent, rusqlite::Error>>>();

        let entry_contents = entry_contents
            .into_iter()
            .map(|entry_content| entry_content.unwrap())
            .collect::<Vec<EntryContent>>();

        let mut classifier = NaiveBayesClassifier::new_classifier(1.0);
        classifier.train(entry_contents);

        Ok(classifier)
    }
    fn new_classifier(alpha: f64) -> NaiveBayesClassifier {
        return NaiveBayesClassifier {
            alpha,
            tokens: HashSet::new(),
            token_liked_counts: HashMap::new(),
            token_disliked_counts: HashMap::new(),
            disliked_entries_count: 0,
            liked_entries_count: 0,
        };
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

    pub fn classify(&self, entry: Entry) -> PossiblyLiked {
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

        return (prob_if_dislike / (prob_if_dislike + prob_if_liked)) < 0.5;
    }

    fn probabilities_of_message(&self, message_tokens: HashSet<&str>) -> (f64, f64) {
        let mut log_prob_if_spam = 0.;
        let mut log_prob_if_ham = 0.;

        for token in self.tokens.iter() {
            let (prob_if_spam, prob_if_ham) = self.probabilites_of_token(&token);

            if message_tokens.contains(token.as_str()) {
                log_prob_if_spam += prob_if_spam.ln();
                log_prob_if_ham += prob_if_ham.ln();
            } else {
                log_prob_if_spam += (1. - prob_if_spam).ln();
                log_prob_if_ham += (1. - prob_if_ham).ln();
            }
        }

        let prob_if_spam = log_prob_if_spam.exp();
        let prob_if_ham = log_prob_if_ham.exp();

        return (prob_if_spam, prob_if_ham);
    }

    fn probabilites_of_token(&self, token: &str) -> (f64, f64) {
        let prob_of_token_spam = (self.token_disliked_counts[token] as f64 + self.alpha)
            / (self.liked_entries_count as f64 + 2. * self.alpha);

        let prob_of_token_ham = (self.token_liked_counts[token] as f64 + self.alpha)
            / (self.liked_entries_count as f64 + 2. * self.alpha);

        return (prob_of_token_spam, prob_of_token_ham);
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

    fn tokenize(lower_case_text: &str) -> HashSet<&str> {
        Regex::new(r"[a-z0-9']+")
            .unwrap()
            .find_iter(lower_case_text)
            .map(|mat| mat.as_str())
            .collect()
    }
}
