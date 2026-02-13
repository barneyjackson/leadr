// Unit tests for leadr-api
// This file is the main entry point for unit tests, organized by module

mod common; // Shared test infrastructure for DB-backed tests
mod unit;

// Re-export all unit test modules
pub use unit::*;

// For unit tests that need database access, use:
// let pool = common::create_test_pool().await;
// let mut tx = pool.begin().await.unwrap();
// ... perform operations on &mut tx ...
// Transaction automatically rolls back when tx is dropped
