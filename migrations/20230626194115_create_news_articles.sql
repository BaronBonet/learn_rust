CREATE TABLE news_articles (
   id SERIAL PRIMARY KEY,
   title TEXT NOT NULL,
   category TEXT NOT NULL,
   date TIMESTAMP NOT NULL,
   url TEXT NOT NULL,
   domain TEXT NOT NULL,
   language TEXT NOT NULL,
   country TEXT NOT NULL
);
