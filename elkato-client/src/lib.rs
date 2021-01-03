//! API for Elkato car sharing system
//!
//! This crate provides a usable API, by scraping HTML pages for information. This is necessary as
//! the system doesn't provide an APIs, and als the HTML code is way older than HTML4. So this
//! crate uses 'nom' to scrape information from the generated pages.

#[cfg(feature = "reqwest")]
mod client;
mod config;
#[cfg(feature = "reqwest")]
mod parser;

#[cfg(feature = "reqwest")]
pub use client::*;
pub use config::*;
