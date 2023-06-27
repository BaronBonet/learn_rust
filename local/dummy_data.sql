INSERT INTO categories (name) values  ('climate change');

INSERT INTO news_articles (title, domain, country, seen_at, url, language)
values ('Test', 'test.com', 'FRA', '2023-01-01 00:00:00', 'https://test.com', 'fr');

INSERT INTO news_article_categories (news_article_id, category_name) values (1, 'climate change');
