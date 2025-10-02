//! AI-based object matching for Bangumi search
//!
//! This module provides intelligent matching between source works from kansou
//! and candidate works from Bangumi using AI semantic understanding.

pub mod matcher;
pub mod types;

// Re-export the main API
#[allow(unused_imports)]
pub use matcher::match_works_with_ai;
#[allow(unused_imports)]
pub use types::*;