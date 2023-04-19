CREATE TYPE LIMIT_LEVEL AS ENUM (
    'KycLevel0',
    'KycLevel1',
    'KycLevel2',
    'KycLevel3',
    'Overall'
);

CREATE TYPE LIMIT_PERIOD AS ENUM (
    'Daily',
    'Weekly',
    'Monthly'
);

CREATE TABLE customer_limits (
    period LIMIT_PERIOD NOT NULL,
    level LIMIT_LEVEL NOT NULL,

    amount NUMERIC NOT NULL,
    currency TEXT NOT NULL,

    customer_id UUID NOT NULL,
    PRIMARY KEY (level, period, customer_id),
    FOREIGN KEY (customer_id)
        REFERENCES customers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE retailer_limits (
    period LIMIT_PERIOD NOT NULL,
    level LIMIT_LEVEL NOT NULL,

    amount NUMERIC NOT NULL,
    currency TEXT NOT NULL,

    retailer_id UUID NOT NULL,
    PRIMARY KEY (level, period, retailer_id),
    FOREIGN KEY (retailer_id)
        REFERENCES retailers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE global_limits (
    period LIMIT_PERIOD NOT NULL,
    level LIMIT_LEVEL NOT NULL,

    amount NUMERIC NOT NULL,
    currency TEXT NOT NULL,

    PRIMARY KEY (level, period)
);

ALTER TABLE customers ADD min_purchase_amount NUMERIC NOT NULL DEFAULT 0;
ALTER TABLE customers ADD min_purchase_currency TEXT NOT NULL DEFAULT 0;

INSERT INTO global_limits VALUES
    ('Daily', 'KycLevel0', 0, 'USD'),
    ('Daily', 'KycLevel1', 0, 'USD'),
    ('Daily', 'KycLevel2', 0, 'USD'),
    ('Daily', 'KycLevel3', 0, 'USD'),
    ('Daily', 'Overall', 0, 'USD'),
    ('Weekly', 'KycLevel0', 0, 'USD'),
    ('Weekly', 'KycLevel1', 0, 'USD'),
    ('Weekly', 'KycLevel2', 0, 'USD'),
    ('Weekly', 'KycLevel3', 0, 'USD'),
    ('Weekly', 'Overall', 0, 'USD'),
    ('Monthly', 'KycLevel0', 0, 'USD'),
    ('Monthly', 'KycLevel1', 0, 'USD'),
    ('Monthly', 'KycLevel2', 0, 'USD'),
    ('Monthly', 'KycLevel3', 0, 'USD'),
    ('Monthly', 'Overall', 0, 'USD');