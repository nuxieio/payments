-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    app_user_id TEXT NOT NULL,          -- The user ID from the client app
    email TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(app_user_id)
);

-- Products table (synced from Apple/Google)
CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,                -- Our internal product ID
    name TEXT NOT NULL,
    description TEXT,
    apple_product_id TEXT,              -- Apple's product identifier
    google_product_id TEXT,             -- Google's product identifier
    type TEXT NOT NULL,                 -- 'subscription' or 'one_time'
    price_usd REAL,                     -- Base price in USD
    duration_days INTEGER,              -- For subscriptions
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(apple_product_id),
    UNIQUE(google_product_id)
);

-- Product entitlement map
CREATE TABLE IF NOT EXISTS product_entitlements (
    product_id TEXT NOT NULL,
    entitlement_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (product_id, entitlement_id),
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);

-- Entitlements table
CREATE TABLE IF NOT EXISTS entitlements (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Subscriptions table
CREATE TABLE IF NOT EXISTS subscriptions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    original_transaction_id TEXT,        -- Original transaction ID from the store
    store_transaction_id TEXT,           -- Current transaction ID from the store
    store TEXT NOT NULL,                 -- 'apple' or 'google'
    purchase_date TIMESTAMP NOT NULL,
    expires_date TIMESTAMP,              -- NULL for lifetime access
    cancellation_date TIMESTAMP,         -- NULL if not cancelled
    renewal_grace_period_expires_date TIMESTAMP, -- For grace period after failed payment
    status TEXT NOT NULL,                -- 'active', 'expired', 'cancelled', 'grace_period', etc.
    auto_renew_status BOOLEAN,
    price_paid REAL,                     -- Actual price paid
    currency TEXT,                       -- Currency code (USD, EUR, etc.)
    is_trial BOOLEAN DEFAULT FALSE,
    is_intro_offer BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    UNIQUE(user_id, product_id, store_transaction_id)
);

-- User entitlements (current active entitlements)
CREATE TABLE IF NOT EXISTS user_entitlements (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    entitlement_id TEXT NOT NULL,
    subscription_id TEXT,                -- Can be NULL for manual grants
    starts_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP,                -- NULL for lifetime access
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (entitlement_id) REFERENCES entitlements(id) ON DELETE CASCADE,
    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id) ON DELETE SET NULL
);

-- Transaction history (keep track of all transactions)
CREATE TABLE IF NOT EXISTS transactions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    subscription_id TEXT NOT NULL,
    store_transaction_id TEXT NOT NULL,
    store TEXT NOT NULL,                 -- 'apple' or 'google'
    type TEXT NOT NULL,                  -- 'initial_purchase', 'renewal', 'cancellation', etc.
    amount REAL,
    currency TEXT,
    transaction_date TIMESTAMP NOT NULL,
    raw_data TEXT,                      -- Store the raw webhook data for debugging
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id) ON DELETE CASCADE
);

-- Store-specific credentials for syncing products and validating receipts
CREATE TABLE IF NOT EXISTS store_credentials (
    id TEXT PRIMARY KEY,
    store TEXT NOT NULL,                 -- 'apple' or 'google'
    app_bundle_id TEXT NOT NULL,         -- Bundle ID / package name
    credentials_json TEXT NOT NULL,      -- Encrypted store credentials
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store, app_bundle_id)
);

-- Create indexes for common queries
CREATE INDEX idx_user_app_user_id ON users(app_user_id);
CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_expires_date ON subscriptions(expires_date);
CREATE INDEX idx_user_entitlements_user_id ON user_entitlements(user_id);
CREATE INDEX idx_user_entitlements_expires_at ON user_entitlements(expires_at);
CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_subscription_id ON transactions(subscription_id);
