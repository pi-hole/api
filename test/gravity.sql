PRAGMA FOREIGN_KEYS = ON;

CREATE TABLE whitelist
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    domain        TEXT UNIQUE NOT NULL,
    enabled       BOOLEAN     NOT NULL DEFAULT 1,
    date_added    INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    date_modified INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    comment       TEXT
);

CREATE TABLE blacklist
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    domain        TEXT UNIQUE NOT NULL,
    enabled       BOOLEAN     NOT NULL DEFAULT 1,
    date_added    INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    date_modified INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    comment       TEXT
);

CREATE TABLE regex
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    domain        TEXT UNIQUE NOT NULL,
    enabled       BOOLEAN     NOT NULL DEFAULT 1,
    date_added    INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    date_modified INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    comment       TEXT
);

CREATE TABLE adlists
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    address       TEXT UNIQUE NOT NULL,
    enabled       BOOLEAN     NOT NULL DEFAULT 1,
    date_added    INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    date_modified INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    comment       TEXT
);

CREATE TABLE gravity
(
    domain TEXT PRIMARY KEY
);

CREATE TABLE info
(
    property TEXT PRIMARY KEY,
    value    TEXT NOT NULL
);

CREATE VIEW vw_whitelist AS
SELECT a.domain
FROM whitelist a
WHERE a.enabled == 1
ORDER BY a.id;

CREATE TRIGGER tr_whitelist_update
    AFTER UPDATE
    ON whitelist
BEGIN
    UPDATE whitelist SET date_modified = (cast(strftime('%s', 'now') as int)) WHERE domain = NEW.domain;
END;

CREATE VIEW vw_blacklist AS
SELECT a.domain
FROM blacklist a
WHERE a.enabled == 1
  AND a.domain NOT IN vw_whitelist
ORDER BY a.id;

CREATE TRIGGER tr_blacklist_update
    AFTER UPDATE
    ON blacklist
BEGIN
    UPDATE blacklist SET date_modified = (cast(strftime('%s', 'now') as int)) WHERE domain = NEW.domain;
END;

CREATE VIEW vw_regex AS
SELECT a.domain
FROM regex a
WHERE a.enabled == 1
ORDER BY a.id;

CREATE TRIGGER tr_regex_update
    AFTER UPDATE
    ON regex
BEGIN
    UPDATE regex SET date_modified = (cast(strftime('%s', 'now') as int)) WHERE domain = NEW.domain;
END;

CREATE VIEW vw_adlists AS
SELECT a.address
FROM adlists a
WHERE a.enabled == 1
ORDER BY a.id;

CREATE TRIGGER tr_adlists_update
    AFTER UPDATE
    ON adlists
BEGIN
    UPDATE adlists SET date_modified = (cast(strftime('%s', 'now') as int)) WHERE address = NEW.address;
END;

CREATE VIEW vw_gravity AS
SELECT a.domain
FROM gravity a
WHERE a.domain NOT IN (SELECT domain from whitelist WHERE enabled == 1);

INSERT INTO whitelist
VALUES (1, 'test.com', 1, 1557712172, 1557712172, NULL),
       (2, 'disabled-white.com', 0, 1557723854, 1557723911, NULL);

INSERT INTO blacklist
VALUES (1, 'example.com', 1, 1557712177, 1557712177, NULL),
       (2, 'disabled-black.com', 0, 1557723854, 1557723864, NULL);

INSERT INTO regex
VALUES (1, '(^|\.)example\.com$', 1, 1557712181, 1557712181, NULL),
       (2, 'disabled\-regex\.com', 0, 1557723854, 1557723872, NULL);

INSERT INTO adlists
VALUES (1, 'https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts', 1, 1557712118, 1557712118,
        'Migrated from /etc/pihole/adlists.list'),
       (2, 'https://mirror1.malwaredomains.com/files/justdomains', 1, 1557712118, 1557712118,
        'Migrated from /etc/pihole/adlists.list');

INSERT INTO gravity
VALUES ('test.com'),
       ('vqubwduhbsd.com'),
       ('vquf4tcdpt22px9l2jqqq.science'),
       ('vqwdsvjygnah.com'),
       ('vqxzysmhsvloijm12fsuswlu.download'),
       ('vr-private-kunden-de.tk'),
       ('vr-private-kundes-de.tk'),
       ('vra.outbrain.com'),
       ('vra4.com'),
       ('vriaj.com');

INSERT INTO info
VALUES ('version', '1');
