-- User login and permission data --
CREATE TABLE users(
  id SERIAL PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  pass TEXT, -- If NULL _all_ passwords considered wrong --
  locked BOOL NOT NULL DEFAULT FALSE,
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

-- Reserve indices and usernames for system/testing users --
INSERT INTO users VALUES(0, 'admin', NULL, false, true);
INSERT INTO users VALUES(-1, 'test-admin', NULL, false, true);
INSERT INTO users VALUES(-2, 'test-user', NULL, false, false);

