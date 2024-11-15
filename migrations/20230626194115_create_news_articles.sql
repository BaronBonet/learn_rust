CREATE TABLE categories (
    name TEXT PRIMARY KEY
);

CREATE TABLE countries (
    iso_alpha_3 VARCHAR(3) PRIMARY KEY
);

CREATE TABLE news_articles (
   id SERIAL PRIMARY KEY,
   title TEXT,
   domain TEXT,
   country_iso_alpha_3 VARCHAR(3) REFERENCES countries(iso_alpha_3),
   seen_at timestamptz,
   url TEXT NOT NULL,
   language TEXT,
   CONSTRAINT unique_article UNIQUE (title, domain, seen_at, country_iso_alpha_3)
);

CREATE TABLE news_article_categories (
     news_article_id INT REFERENCES news_articles(id),
     category_name TEXT REFERENCES categories(name),
     PRIMARY KEY (news_article_id, category_name)
);
