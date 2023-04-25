CREATE TABLE partners(
    id UUID PRIMARY KEY,

    approved_at TIMESTAMP WITH TIME ZONE,
    approved APPROVAL,

    account_id UUID UNIQUE,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE retailer_partners (
    retailer_id UUID NOT NULL,
    partner_id UUID NOT NULL,
    
    PRIMARY KEY (retailer_id, partner_id),
    FOREIGN KEY (retailer_id)
        REFERENCES retailers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (partner_id)
        REFERENCES partners (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE payment_methods (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE retailer_fees (
    payment_method_id UUID NOT NULL,
    retailer_id UUID NOT NULL,

    retailer_fee NUMERIC NOT NULL,
    consumer_fee NUMERIC NOT NULL,
    exchange_spread NUMERIC NOT NULL,
    exchange_spread_stable_coin NUMERIC NOT NULL,
    min_transaction_fee NUMERIC NOT NULL,
    base_additional_fixed_fee_amount NUMERIC NOT NULL,
    base_additional_fixed_fee_currency TEXT NOT NULL,

    PRIMARY KEY (payment_method_id, retailer_id),
    FOREIGN KEY (payment_method_id)
        REFERENCES payment_methods (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (retailer_id)
        REFERENCES retailers (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE partner_fees (
    payment_method_id UUID NOT NULL,
    partner_id UUID NOT NULL,

    referral_partner_fee NUMERIC NOT NULL,
    additional_fixed_fee_amount NUMERIC NOT NULL,
    additional_fixed_fee_currency TEXT NOT NULL,

    PRIMARY KEY (payment_method_id, partner_id),
    FOREIGN KEY (payment_method_id)
        REFERENCES payment_methods (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (partner_id)
        REFERENCES partners (id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);