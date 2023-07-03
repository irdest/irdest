-- The initial SQL migration code

--- Create a table for registered clients/ applications
CREATE TABLE IF NOT EXISTS clients
(
        id              INTEGER PRIMARY KEY NOT NULL,
        token_hash      TEXT                NOT NULL,
        last_connection TEXT                NOT NULL,
);


--- Create a table for local addresses, which references a single
--- client, meaning that any local client can have many different
--- addresses associated, but never two clients the same address.
CREATE TABLE IF NOT EXISTS local_addresses
(
        id              INTEGER PRIMARY KEY NOT NULL,
        content         TEXT                NOT NULL,
        client_id       INTEGER             NOT NULL,
        FOREIGN         KEY(client_id) REFERENCES clients(id),
);
