DROP TABLE IF EXISTS global_limits;
DROP TABLE IF EXISTS retailer_limits;
DROP TABLE IF EXISTS customer_limits;
DROP TYPE IF EXISTS LIMIT_LEVEL;
DROP TYPE IF EXISTS LIMIT_PERIOD;

ALTER TABLE customers DROP COLUMN min_purchase_amount;
ALTER TABLE customers DROP COLUMN min_purchase_currency;