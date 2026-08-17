//! # cedar-json-parser
//!
//! A formally verified JSON parser (RFC 8259) built with [Verus](https://github.com/verus-lang/verus).
//!
//! This crate provides a single entry point, [`parse_json`], which tokenizes and parses
//! a byte slice as a JSON document. The implementation is proven correct against the
//! RFC 8259 grammar using Verus's SMT-backed verification. When reading the RFC 8259 specification,
//! we interpret the following "SHOULD" in "the names within an object SHOULD be unique" as a
//! "MUST"; we enforce uniqueness of keys in objects.
//!
//! ## Verified properties
//!
//! - **Soundness**: if `parse_json` returns `Ok`, the result faithfully represents the
//!   input according to the RFC 8259 grammar (correct tokenization, escape decoding,
//!   UTF-8 validation, duplicate key rejection).
//! - **Safety**: the parser never panics on arbitrary inputs, unless an out-of-memory or a
//!   stack overflow occurs. The tokenizer does limit nesting to [`tokenizer::MAX_NESTING_DEPTH`]
//!   to avoid such cases.
//! - **Tokenizer completeness**: given a valid string of JSON tokens, the tokenizer will generate
//!   Ok(..) with a list of tokens.
//!
//! ## Unverified
//! - **Parser completeness**: we do not have a proof that the parser will succeed for every valid
//!   input string.
//!
//! ## Usage
//!
//! ```
//! use cedar_json_parser::{parse_json, JsonValue};
//!
//! let input = br#"{"key": [1, 2, 3]}"#;
//! match parse_json(input) {
//!     Ok(value) => { /* process JsonValue tree */ }
//!     Err(e) => { /* handle ParseJsonError */ }
//! }
//! ```

// Verus `use` statements and spec functions are erased during normal compilation,
// which causes spurious unused-import and dead-code warnings.
#![allow(unused_imports, dead_code)]

#[allow(non_snake_case)]
pub(crate) mod byte_specs;
pub(crate) mod decimal;
pub(crate) mod dedup;
pub(crate) mod escape;
pub(crate) mod fuel_mono;
pub mod json_spec;
mod parser;
pub(crate) mod tokenizer;
pub(crate) mod utf8_validation;

pub use json_spec::JsonValueSpec;
pub use parser::{parse_json, JsonValue, ObjectEntry, ParseError, ParseJsonError};
pub use tokenizer::{Token, TokenKind, TokenizeError, MAX_NESTING_DEPTH};
