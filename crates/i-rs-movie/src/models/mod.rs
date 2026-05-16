use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub name: String,
    pub year: Option<i32>,
    pub director: Option<String>,
    pub watched: bool,
    pub rating: Option<f32>,
    pub review: Vec<String>,
    pub release_date: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct MovieStore {
    pub movies: BTreeMap<String, Movie>,
}


#[allow(dead_code)]
impl MovieStore {
    pub fn add_entry(&mut self, movie: Movie) {
        self.movies.insert(movie.name.clone(), movie);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<Movie> {
        self.movies.remove(name)
    }

    pub fn get_entry(&self, name: &str) -> Option<&Movie> {
        self.movies.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut Movie> {
        self.movies.get_mut(name)
    }

    pub fn get_all_movies(&self) -> Vec<&Movie> {
        self.movies.values().collect()
    }

    pub fn get_watched_movies(&self) -> Vec<&Movie> {
        self.movies.values().filter(|m| m.watched).collect()
    }

    pub fn get_unwatched_movies(&self) -> Vec<&Movie> {
        self.movies.values().filter(|m| !m.watched).collect()
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&Movie> {
        self.movies
            .values()
            .filter(|m| m.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn movie_stats(&self) -> MovieStats {
        let total = self.movies.len();
        let watched = self.movies.values().filter(|m| m.watched).count();
        let unwatched = total - watched;
        let rated: Vec<f32> = self.movies.values().filter_map(|m| m.rating).collect();
        let avg_rating = if rated.is_empty() {
            None
        } else {
            Some(rated.iter().sum::<f32>() / rated.len() as f32)
        };

        MovieStats {
            total,
            watched,
            unwatched,
            avg_rating,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieStats {
    pub total: usize,
    pub watched: usize,
    pub unwatched: usize,
    pub avg_rating: Option<f32>,
}

#[derive(Tabled)]
pub struct MovieRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "YEAR")]
    year: String,
    #[tabled(rename = "DIRECTOR")]
    director: String,
    #[tabled(rename = "WATCHED")]
    watched: String,
    #[tabled(rename = "RATING")]
    rating: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl MovieRow {
    pub fn from_movie(movie: &Movie) -> Self {
        Self {
            name: movie.name.clone(),
            year: movie.year.map_or_else(|| "-".to_string(), |y| y.to_string()),
            director: movie.director.clone().unwrap_or_else(|| "-".to_string()),
            watched: if movie.watched { "✓" } else { "-" }.to_string(),
            rating: movie.rating.map_or_else(|| "-".to_string(), |r| format!("{r:.1}")),
            tags: if movie.tags.is_empty() {
                "-".to_string()
            } else {
                movie.tags.join(", ")
            },
        }
    }
}

#[allow(dead_code)]
pub trait HasTags {
    fn get_tags(&self) -> &[String];
    fn has_tag(&self, tag: &str) -> bool {
        self.get_tags().iter().any(|t| t == tag)
    }
}

#[allow(dead_code)]
impl HasTags for Movie {
    fn get_tags(&self) -> &[String] {
        &self.tags
    }
}
