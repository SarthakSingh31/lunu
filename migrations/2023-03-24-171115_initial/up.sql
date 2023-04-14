CREATE TYPE KYC_LEVEL AS ENUM ('Level0', 'Level1', 'Level2', 'Level3');

CREATE TYPE SCOPE AS ENUM (
    'Public',
    'Customer',
    'Merchant',
    'Partner',
    'Admin'
);

-- Authentication
CREATE TABLE accounts(
    id UUID PRIMARY KEY DEFAULT GEN_RANDOM_UUID(),
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    blocked BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE scopes(
    account_id UUID NOT NULL,
    scope SCOPE NOT NULL,
    PRIMARY KEY (account_id, scope),
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE email_login_intents(
    id UUID PRIMARY KEY DEFAULT GEN_RANDOM_UUID(),
    account_id UUID NOT NULL,
    pass_key TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE new_pass_login_intents(
    id TEXT PRIMARY KEY,
    account_id UUID NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE password_login(
    account_id UUID PRIMARY KEY,
    hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE sessions(
    token TEXT PRIMARY KEY,
    account_id UUID NOT NULL,
    password_login BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

-- Customer Data
CREATE TABLE merchants(
    id UUID PRIMARY KEY,

    addr_line_1 TEXT NOT NULL,
    addr_line_2 TEXT NOT NULL,
    country TEXT NOT NULL,

    approved_at TIMESTAMP WITH TIME ZONE,
    approved BOOLEAN NOT NULL DEFAULT false,

    account_id UUID UNIQUE,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE customers(
    id UUID PRIMARY KEY,

    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    
    kyc_level KYC_LEVEL NOT NULL DEFAULT 'Level0',

    account_id UUID UNIQUE,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE transactions(
    id UUID PRIMARY KEY,

    merchant_id UUID,
    retailer_transaction_id TEXT,
    retailer_customer_id TEXT,

    source_account_wallet UUID NOT NULL,
    dest_account_wallet UUID NOT NULL,

    kind TEXT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    payment_method TEXT NOT NULL,
    crypto_currency_type TEXT NOT NULL,
    crypto_network TEXT NOT NULL,
    crypto_amount MONEY NOT NULL,
    fiat_type TEXT NOT NULL,
    fiat_amount MONEY NOT NULL,
    exchange_rate MONEY NOT NULL,
    dest_crypto_address INTEGER NOT NULL,
    transcation_hash INTEGER NOT NULL,
    payment_gateway_fee MONEY NOT NULL,
    exchange_spread_fee MONEY NOT NULL,
    partner_fee MONEY NOT NULL,
    status INTEGER NOT NULL,
    FOREIGN KEY (merchant_id)
        REFERENCES merchants (id)
            ON UPDATE CASCADE
            ON DELETE SET NULL,
    FOREIGN KEY (source_account_wallet)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (dest_account_wallet)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);