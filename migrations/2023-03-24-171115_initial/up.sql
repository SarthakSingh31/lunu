CREATE TYPE KYC_LEVEL AS ENUM ('Level1', 'Level2', 'Level3');

CREATE TABLE accounts(
  id UUID PRIMARY KEY DEFAULT GEN_RANDOM_UUID(),

  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  approved_at TIMESTAMP WITH TIME ZONE,
  approved BOOLEAN NOT NULL DEFAULT false,
  blocked BOOLEAN NOT NULL DEFAULT false,
  
  kyc_level KYC_LEVEL NOT NULL,

  addr_line_1 TEXT NOT NULL,
  addr_line_2 TEXT NOT NULL,
  country TEXT NOT NULL
);

CREATE TABLE merchants(
    id UUID PRIMARY KEY,

    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL,

    account_id UUID UNIQUE,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE SET NULL
);

CREATE TABLE customers(
    id UUID PRIMARY KEY,

    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL,

    total_risk REAL NOT NULL,

    personal_limit MONEY,
    personal_kyc_limit MONEY,

    account_id UUID UNIQUE,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE SET NULL
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