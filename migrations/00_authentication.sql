-- User login and permission data --
CREATE TABLE users(
  id SERIAL PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  pass TEXT, -- If NULL the user is disabled --
  admin BOOL NOT NULL DEFAULT FALSE
);

-- Session storage --
CREATE TABLE sessions(
  id SERIAL PRIMARY KEY,
  userid INTEGER NOT NULL,
  key TEXT NOT NULL UNIQUE,
  until TIMESTAMP NOT NULL,

  FOREIGN KEY (userid) REFERENCES users
);
