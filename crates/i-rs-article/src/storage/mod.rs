use crate::models::{Article, ArticleStore, ReadStatus};


i_rs_core::create_store!(ArticleStore, "article");


pub fn add_article(store: &mut ArticleStore, article: Article) {
    store.articles.insert(article.name.clone(), article);
}

pub fn remove_article(store: &mut ArticleStore, name: &str) -> Option<Article> {
    store.articles.remove(name)
}

pub fn get_article<'a>(store: &'a ArticleStore, name: &str) -> Option<&'a Article> {
    store.articles.get(name)
}

pub fn get_article_mut<'a>(store: &'a mut ArticleStore, name: &str) -> Option<&'a mut Article> {
    store.articles.get_mut(name)
}

pub fn filter_by_tag_and_status<'a>(
    store: &'a ArticleStore,
    tag: Option<&str>,
    status: Option<ReadStatus>,
) -> Vec<&'a Article> {
    let articles: Vec<&Article> = if let Some(tag) = tag {
        store
            .articles
            .values()
            .filter(|a| a.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.articles.values().collect()
    };

    if let Some(status) = status {
        articles.into_iter().filter(|a| a.status == status).collect()
    } else {
        articles
    }
}

pub fn get_stats(store: &ArticleStore) -> ArticleStats {
    let total = store.articles.len();
    let unread = store.articles.values().filter(|a| a.status == ReadStatus::Unread).count();
    let reading = store.articles.values().filter(|a| a.status == ReadStatus::Reading).count();
    let read = store.articles.values().filter(|a| a.status == ReadStatus::Read).count();

    ArticleStats {
        total,
        unread,
        reading,
        read,
    }
}

#[derive(Debug, Clone)]
pub struct ArticleStats {
    pub total: usize,
    pub unread: usize,
    pub reading: usize,
    pub read: usize,
}
