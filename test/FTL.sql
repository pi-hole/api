PRAGMA FOREIGN_KEYS = ON;

-- BEGIN SCHEMA

CREATE TABLE ftl
(
    id    INTEGER PRIMARY KEY NOT NULL,
    value BLOB                NOT NULL
);

CREATE TABLE counters
(
    id    INTEGER PRIMARY KEY NOT NULL,
    value INTEGER             NOT NULL
);

CREATE TABLE network
(
    id         INTEGER PRIMARY KEY NOT NULL,
    ip         TEXT                NOT NULL,
    hwaddr     TEXT                NOT NULL,
    interface  TEXT                NOT NULL,
    name       TEXT,
    firstSeen  INTEGER             NOT NULL,
    lastQuery  INTEGER             NOT NULL,
    numQueries INTEGER             NOT NULL,
    macVendor  TEXT
);

CREATE TABLE queries
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    type      INTEGER NOT NULL,
    status    INTEGER NOT NULL,
    domain    TEXT    NOT NULL,
    client    TEXT    NOT NULL,
    forward   TEXT
);

CREATE INDEX idx_queries_timestamps ON queries (timestamp);

-- BEGIN TEST DATA

INSERT INTO ftl
VALUES
    -- Version
    (0, 3),
    -- Last query timestamp
    (1, 177180),
    -- First counter timestamp
    (2, 0);

INSERT INTO counters
VALUES
    -- Total queries
    (0, 5980376),
    -- Blocked queries
    (1, 19382);

INSERT INTO network
VALUES (1,
        '10.1.1.1',
        '00:00:00:00:00:00',
        'eth0',
        'gateway',
        1546832160,
        1547002023,
        6,
        'Fantasy Devices Inc');

INSERT INTO queries
VALUES (1, 0, 6, 3, '1.1.1.10.in-addr.arpa', '127.0.0.1', NULL),
       (2, 0, 6, 2, '4.4.8.8.in-addr.arpa', '127.0.0.1', '8.8.4.4'),
       (3, 164431, 1, 2, '0.ubuntu.pool.ntp.org', '127.0.0.1', '8.8.4.4'),
       (4, 164431, 2, 2, '0.ubuntu.pool.ntp.org', '127.0.0.1', '8.8.4.4'),
       (5, 164433, 1, 2, '1.ubuntu.pool.ntp.org', '127.0.0.1', '8.8.4.4'),
       (6, 164433, 2, 2, '1.ubuntu.pool.ntp.org', '127.0.0.1', '8.8.4.4'),
       (7, 164475, 1, 2, 'github.com', '127.0.0.1', '8.8.4.4'),
       (8, 164475, 2, 2, 'github.com', '127.0.0.1', '8.8.4.4'),
       (9, 164478, 1, 3, 'github.com', '127.0.0.1', NULL),
       (10, 164478, 2, 3, 'github.com', '127.0.0.1', NULL),
       (11, 164483, 1, 3, 'github.com', '127.0.0.1', NULL),
       (12, 164483, 2, 3, 'github.com', '127.0.0.1', NULL),
       (13, 164580, 6, 2, '4.4.8.8.in-addr.arpa', '127.0.0.1', '8.8.4.4'),
       (14, 164583, 1, 2, 'google.com', '10.1.1.1', '8.8.4.4'),
       (15, 164636, 1, 2, 'ftl.pi-hole.net', '127.0.0.1', '8.8.4.4'),
       (16, 164636, 2, 2, 'ftl.pi-hole.net', '127.0.0.1', '8.8.4.4'),
       (17, 164638, 1, 2, 'github.com', '127.0.0.1', '8.8.8.8'),
       (18, 164638, 2, 2, 'github.com', '127.0.0.1', '8.8.8.8'),
       (19, 164638, 1, 3, 'ftl.pi-hole.net', '127.0.0.1', NULL),
       (20, 164638, 2, 3, 'ftl.pi-hole.net', '127.0.0.1', NULL),
       (21, 164642, 6, 3, '1.1.1.10.in-addr.arpa', '127.0.0.1', NULL),
       (22, 164642, 6, 2, '4.4.8.8.in-addr.arpa', '127.0.0.1', '8.8.8.8'),
       (23, 164642, 6, 2, '8.8.8.8.in-addr.arpa', '127.0.0.1', '8.8.8.8'),
       (24, 164642, 1, 3, 'ftl.pi-hole.net', '127.0.0.1', NULL),
       (25, 164642, 2, 3, 'ftl.pi-hole.net', '127.0.0.1', NULL),
       (26, 164700, 6, 3, '1.1.1.10.in-addr.arpa', '127.0.0.1', NULL),
       (27, 164700, 6, 2, '4.4.8.8.in-addr.arpa', '127.0.0.1', '8.8.4.4'),
       (28, 164700, 6, 2, '8.8.8.8.in-addr.arpa', '127.0.0.1', '8.8.4.4'),
       (29, 165360, 6, 3, '1.1.1.10.in-addr.arpa', '127.0.0.1', NULL),
       (30, 165360, 6, 3, '4.4.8.8.in-addr.arpa', '127.0.0.1', NULL),
       (31, 165360, 6, 3, '8.8.8.8.in-addr.arpa', '127.0.0.1', NULL),
       (32, 165540, 1, 2, 'github.com', '127.0.0.1', '8.8.4.4'),
       (33, 165540, 2, 2, 'github.com', '127.0.0.1', '8.8.4.4'),
       (34, 165544, 1, 3, 'github.com', '127.0.0.1', NULL),
       (35, 165544, 2, 3, 'github.com', '127.0.0.1', NULL),
       (36, 168960, 6, 3, '1.1.1.10.in-addr.arpa', '127.0.0.1', NULL),
       (37, 168960, 6, 3, '4.4.8.8.in-addr.arpa', '127.0.0.1', NULL),
       (38, 168961, 6, 3, '8.8.8.8.in-addr.arpa', '127.0.0.1', NULL),
       (39, 172560, 6, 3, '1.1.1.10.in-addr.arpa', '127.0.0.1', NULL),
       (40, 172560, 6, 3, '4.4.8.8.in-addr.arpa', '127.0.0.1', NULL),
       (41, 172560, 6, 3, '8.8.8.8.in-addr.arpa', '127.0.0.1', NULL),
       (42, 174179, 1, 2, '3.ubuntu.pool.ntp.org', '127.0.0.1', '8.8.4.4'),
       (43, 174179, 2, 2, '3.ubuntu.pool.ntp.org', '127.0.0.1', '8.8.4.4'),
       (44, 174197, 1, 2, '0.ubuntu.pool.ntp.org', '127.0.0.1', '8.8.4.4'),
       (45, 174197, 2, 2, '0.ubuntu.pool.ntp.org', '127.0.0.1', '8.8.4.4'),
       (46, 174201, 1, 3, '0.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (47, 174201, 2, 3, '0.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (48, 174207, 1, 2, '1.ubuntu.pool.ntp.org', '127.0.0.1', '8.8.4.4'),
       (49, 174207, 2, 2, '1.ubuntu.pool.ntp.org', '127.0.0.1', '8.8.4.4'),
       (50, 176160, 6, 3, '1.1.1.10.in-addr.arpa', '127.0.0.1', NULL),
       (51, 176160, 6, 3, '4.4.8.8.in-addr.arpa', '127.0.0.1', NULL),
       (52, 176160, 6, 3, '8.8.8.8.in-addr.arpa', '127.0.0.1', NULL),
       (53, 177157, 1, 0, '0.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (54, 177157, 2, 0, '0.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (55, 177157, 1, 0, '0.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (56, 177157, 2, 0, '0.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (57, 177157, 1, 0, '0.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (58, 177157, 2, 0, '0.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (59, 177157, 1, 0, '0.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (60, 177157, 2, 0, '0.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (61, 177158, 1, 0, '1.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (62, 177158, 2, 0, '1.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (63, 177158, 1, 0, '1.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (64, 177158, 2, 0, '1.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (65, 177158, 1, 0, '1.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (66, 177158, 2, 0, '1.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (67, 177158, 1, 0, '1.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (68, 177158, 2, 0, '1.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (69, 177159, 1, 0, '2.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (70, 177159, 2, 0, '2.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (71, 177159, 1, 0, '2.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (72, 177159, 2, 0, '2.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (73, 177159, 1, 0, '2.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (74, 177159, 2, 0, '2.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (75, 177159, 1, 0, '2.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (76, 177159, 2, 0, '2.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (77, 177160, 1, 0, '3.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (78, 177160, 2, 0, '3.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (79, 177160, 1, 0, '3.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (80, 177160, 2, 0, '3.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (81, 177160, 1, 0, '3.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (82, 177160, 2, 0, '3.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (83, 177160, 1, 0, '3.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (84, 177160, 2, 0, '3.ubuntu.pool.ntp.org', '127.0.0.1', NULL),
       (85, 177161, 1, 0, 'ntp.ubuntu.com', '127.0.0.1', NULL),
       (86, 177161, 2, 0, 'ntp.ubuntu.com', '127.0.0.1', NULL),
       (87, 177161, 1, 0, 'ntp.ubuntu.com', '127.0.0.1', NULL),
       (88, 177161, 2, 0, 'ntp.ubuntu.com', '127.0.0.1', NULL),
       (89, 177161, 1, 0, 'ntp.ubuntu.com', '127.0.0.1', NULL),
       (90, 177161, 2, 0, 'ntp.ubuntu.com', '127.0.0.1', NULL),
       (91, 177161, 1, 0, 'ntp.ubuntu.com', '127.0.0.1', NULL),
       (92, 177161, 2, 0, 'ntp.ubuntu.com', '127.0.0.1', NULL),
       (93, 177180, 6, 3, '1.1.1.10.in-addr.arpa', '127.0.0.1', NULL),
       (94, 177180, 6, 2, '4.4.8.8.in-addr.arpa', '127.0.0.1', '8.8.4.4');
