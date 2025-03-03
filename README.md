# Nuxie Payments

An open source payments service for handling Apple and Google mobile app subscriptions. Kind of like RevenueCat, but free.

## Features

- Subscribes to and processes webhook events from Apple App Store and Google Play
- Manages user subscriptions and entitlements
- Provides a REST API for accessing subscription data
- Stores everything in a SQLite database
- Handles subscription lifecycle (purchase, renewal, cancellation, expiration, refunds)
- Maps store products to app entitlements

## Tech Stack

- **Rust**: Core programming language
- **Axum**: Web framework
- **SQLite**: Database
- **SQLx**: Database library and query builder
- **Tokio**: Async runtime

## API Endpoints

### User Endpoints

- `GET /api/users`: List all users
- `POST /api/users`: Create a new user
- `GET /api/users/:user_id`: Get user details
- `PUT /api/users/:user_id`: Update user details
- `DELETE /api/users/:user_id`: Delete a user
- `GET /api/users/app_id/:app_user_id`: Get user by app-specific ID
- `GET /api/users/:user_id/subscriptions`: Get all user subscriptions
- `GET /api/users/:user_id/subscriptions/active`: Get active user subscriptions

### Product Endpoints

- `GET /api/products`: List all products
- `POST /api/products`: Create a new product
- `GET /api/products/:product_id`: Get product details
- `PUT /api/products/:product_id`: Update product details
- `DELETE /api/products/:product_id`: Delete a product
- `POST /api/products/:product_id/entitlements`: Add entitlement to product
- `DELETE /api/products/:product_id/entitlements/:entitlement_id`: Remove entitlement from product

### Entitlement Endpoints

- `POST /api/entitlements`: Create a new entitlement
- `GET /api/users/:user_id/entitlements`: Get user's entitlements
- `GET /api/users/:user_id/entitlements/:entitlement_id`: Check specific entitlement access
- `POST /api/entitlements/grant`: Grant entitlement to user
- `POST /api/users/:user_id/entitlements/:entitlement_id/revoke`: Revoke entitlement

### Subscription Endpoints

- `GET /api/subscriptions`: List all subscriptions
- `GET /api/subscriptions/:subscription_id`: Get subscription details
- `POST /api/subscriptions/:subscription_id/cancel`: Cancel a subscription
- `POST /api/subscriptions/:subscription_id/refund`: Refund a subscription

### Webhook Endpoints

- `POST /webhooks/apple`: Apple App Store Server Notifications webhook
- `POST /webhooks/google`: Google Play Real-time Developer Notifications webhook

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
- SQLite

### Installation

1. Clone the repository:

```bash
git clone https://github.com/yourusername/subscription-backend.git
cd subscription-backend
```

2. Create a `.env` file with the following variables:

```
DATABASE_URL=sqlite:./data/subscription.db
HOST=127.0.0.1
PORT=3000
JWT_SECRET=your-jwt-secret
LOG_LEVEL=info
ENVIRONMENT=development
WEBHOOK_SIGNATURE_SECRET=your-webhook-signature-secret
```

3. Build and run the application:

```bash
cargo build --release
cargo run --release
```

## Setup for Production

### Apple App Store

1. Set up Server-to-Server Notifications in App Store Connect
2. Configure the webhook URL to point to `/webhooks/apple`
3. Store your App Store Connect API key and shared secret in the `store_credentials` table

### Google Play

1. Set up Real-time Developer Notifications in Google Play Console
2. Configure the webhook URL to point to `/webhooks/google`
3. Store your Google Play service account credentials in the `store_
