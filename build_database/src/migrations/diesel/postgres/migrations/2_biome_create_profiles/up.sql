

CREATE TABLE IF NOT EXISTS user_profile (
  user_id      TEXT        PRIMARY KEY,
  subject      TEXT        NOT NULL UNIQUE,
  name         TEXT,
  given_name   TEXT,
  family_name  TEXT,
  email        TEXT,
  picture      TEXT
);
