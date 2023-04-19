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