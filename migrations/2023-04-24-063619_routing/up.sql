CREATE TYPE PROFILE_INDEX AS ENUM (
    '0',
    '1',
    '2'
);

CREATE TABLE payment_gateways (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE customer_payment_gateway_routing (
    idx PROFILE_INDEX NOT NULL,
    customer_id UUID NOT NULL,
    selected UUID NOT NULL,
    amount NUMERIC NOT NULL DEFAULT 0,
    currency TEXT NOT NULL,
    PRIMARY KEY (idx, customer_id),
    FOREIGN KEY (customer_id)
        REFERENCES customers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (selected)
        REFERENCES payment_gateways (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE retailer_payment_gateway_routing (
    idx PROFILE_INDEX NOT NULL,
    retailer_id UUID NOT NULL,
    selected UUID NOT NULL,
    amount NUMERIC NOT NULL DEFAULT 0,
    currency TEXT NOT NULL,
    PRIMARY KEY (idx, retailer_id),
    FOREIGN KEY (retailer_id)
        REFERENCES retailers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (selected)
        REFERENCES payment_gateways (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE global_payment_gateway_routing (
    idx PROFILE_INDEX PRIMARY KEY,
    selected UUID,
    amount NUMERIC,
    currency TEXT,
    FOREIGN KEY (selected)
        REFERENCES payment_gateways (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE custody_providers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE customer_custody_provider_routing (
    idx PROFILE_INDEX NOT NULL,
    customer_id UUID NOT NULL,
    selected UUID NOT NULL,
    amount NUMERIC NOT NULL DEFAULT 0,
    currency TEXT NOT NULL,
    PRIMARY KEY (idx, customer_id),
    FOREIGN KEY (customer_id)
        REFERENCES customers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (selected)
        REFERENCES custody_providers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE retailer_custody_provider_routing (
    idx PROFILE_INDEX NOT NULL,
    retailer_id UUID NOT NULL,
    selected UUID NOT NULL,
    amount NUMERIC NOT NULL DEFAULT 0,
    currency TEXT NOT NULL,
    PRIMARY KEY (idx, retailer_id),
    FOREIGN KEY (retailer_id)
        REFERENCES retailers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (selected)
        REFERENCES custody_providers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE global_custody_provider_routing (
    idx PROFILE_INDEX PRIMARY KEY,
    selected UUID,
    amount NUMERIC,
    currency TEXT,
    FOREIGN KEY (selected)
        REFERENCES custody_providers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE exchange_providers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE customer_exchange_provider_routing (
    idx PROFILE_INDEX NOT NULL,
    customer_id UUID NOT NULL,
    selected UUID NOT NULL,
    amount NUMERIC NOT NULL DEFAULT 0,
    currency TEXT NOT NULL,
    PRIMARY KEY (idx, customer_id),
    FOREIGN KEY (customer_id)
        REFERENCES customers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (selected)
        REFERENCES exchange_providers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE retailer_exchange_provider_routing (
    idx PROFILE_INDEX NOT NULL,
    retailer_id UUID NOT NULL,
    selected UUID NOT NULL,
    amount NUMERIC NOT NULL DEFAULT 0,
    currency TEXT NOT NULL,
    PRIMARY KEY (idx, retailer_id),
    FOREIGN KEY (retailer_id)
        REFERENCES retailers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (selected)
        REFERENCES exchange_providers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE global_exchange_provider_routing (
    idx PROFILE_INDEX PRIMARY KEY,
    selected UUID,
    amount NUMERIC,
    currency TEXT,
    FOREIGN KEY (selected)
        REFERENCES exchange_providers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

INSERT INTO global_payment_gateway_routing VALUES ('0', NULL, NULL, NULL), ('1', NULL, NULL, NULL), ('2', NULL, NULL, NULL);
INSERT INTO global_custody_provider_routing VALUES ('0', NULL, NULL, NULL), ('1', NULL, NULL, NULL), ('2', NULL, NULL, NULL);
INSERT INTO global_exchange_provider_routing VALUES ('0', NULL, NULL, NULL), ('1', NULL, NULL, NULL), ('2', NULL, NULL, NULL);