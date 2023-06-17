use chrono::NaiveDate;

pub struct NewsArticle {
   title: String,
     body: String,
     date: NaiveDate,
    url: String,
}

impl NewsArticle {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn date(&self) -> &NaiveDate {
        &self.date
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn new(title: String, body: String, date: NaiveDate, url: String) -> Self {
        Self {
            title,
            body,
            date,
            url,
        }
    }
}
