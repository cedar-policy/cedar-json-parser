//! Integration tests using the JSONTestSuite (https://github.com/nst/JSONTestSuite).
//!
//! File naming convention:
//!   y_*  — content MUST be accepted (valid JSON per RFC 8259)
//!   n_*  — content MUST be rejected (invalid JSON)
//!   i_*  — implementation-defined (either accept or reject is fine)

use cedar_json_parser::{parse_json, ParseError, ParseJsonError, TokenizeError};
use std::fs;
use std::path::Path;

fn format_error(e: &ParseJsonError) -> String {
    match e {
        ParseJsonError::Tokenize { err } => match err {
            TokenizeError::UnexpectedEof { pos } => {
                format!("TokenizeError::UnexpectedEof at {pos}")
            }
            TokenizeError::InvalidNumber { pos } => {
                format!("TokenizeError::InvalidNumber at {pos}")
            }
            TokenizeError::InvalidEscape { pos } => {
                format!("TokenizeError::InvalidEscape at {pos}")
            }
            TokenizeError::UnexpectedToken { pos } => {
                format!("TokenizeError::UnexpectedToken at {pos}")
            }
            TokenizeError::NestingTooDeep { pos } => {
                format!("TokenizeError::NestingTooDeep at {pos}")
            }
        },
        ParseJsonError::Parse { err } => match err {
            ParseError::UnexpectedToken { pos } => {
                format!("ParseError::UnexpectedToken at {pos}")
            }
            ParseError::InvalidEscape { pos } => {
                format!("ParseError::InvalidEscape at {pos}")
            }
            ParseError::DuplicateKey {
                first_pos,
                second_pos,
            } => {
                format!("ParseError::DuplicateKey at {first_pos}, {second_pos}")
            }
        },
    }
}

const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/json_test_suite");

#[test]
fn json_test_suite() {
    // Some test files have extreme nesting (100K+ levels) that overflows the
    // default thread stack. Spawn with a generous stack to handle them.
    let builder = std::thread::Builder::new()
        .name("json_test_suite".into())
        .stack_size(64 * 1024 * 1024); // 64 MB

    let handler = builder
        .spawn(json_test_suite_inner)
        .expect("failed to spawn test thread");

    handler.join().expect("test thread panicked");
}

fn json_test_suite_inner() {
    let dir = Path::new(TEST_DATA_DIR);
    assert!(
        dir.is_dir(),
        "Test data directory not found: {TEST_DATA_DIR}"
    );

    // No files need to be skipped — deep nesting is rejected gracefully
    // by the tokenizer's NestingTooDeep error (MAX_NESTING_DEPTH = 1024).
    let skip: &[&str] = &[];

    // y_ files that we intentionally reject because our parser is stricter than
    // the RFC requires. RFC 8259 §4 says keys SHOULD be unique; we enforce MUST.
    // These are treated as implementation-defined (like i_ files).
    let y_strict_reject: &[&str] = &[
        "y_object_duplicated_key.json",
        "y_object_duplicated_key_and_value.json",
    ];

    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("failed to read test data directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut y_pass = 0u32;
    let mut y_fail = 0u32;
    let mut n_pass = 0u32;
    let mut n_fail = 0u32;
    let mut i_accept = 0u32;
    let mut i_reject = 0u32;
    let mut failures: Vec<String> = Vec::new();

    for entry in &entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if skip.iter().any(|s| *s == &*name) {
            continue;
        }

        let content = fs::read(&path).unwrap_or_else(|e| {
            panic!("failed to read {name}: {e}");
        });

        let result = parse_json(&content);

        if name.starts_with("y_") {
            if y_strict_reject.iter().any(|s| *s == &*name) {
                // Treated as implementation-defined (we intentionally reject these)
                match result {
                    Ok(_) => i_accept += 1,
                    Err(_) => i_reject += 1,
                }
            } else {
                match result {
                    Ok(_) => y_pass += 1,
                    Err(e) => {
                        y_fail += 1;
                        failures.push(format!(
                            "FAIL (should accept): {name} — {}",
                            format_error(&e)
                        ));
                    }
                }
            }
        } else if name.starts_with("n_") {
            match result {
                Err(_) => n_pass += 1,
                Ok(_) => {
                    n_fail += 1;
                    failures.push(format!("FAIL (should reject): {name}"));
                }
            }
        } else if name.starts_with("i_") {
            match result {
                Ok(_) => i_accept += 1,
                Err(_) => i_reject += 1,
            }
        }
    }

    println!();
    println!("=== JSONTestSuite Results ===");
    println!("y_ (must accept):  {y_pass} passed, {y_fail} failed");
    println!("n_ (must reject):  {n_pass} passed, {n_fail} failed");
    println!("i_ (impl-defined): {i_accept} accepted, {i_reject} rejected");
    println!("Total files: {}", entries.len());

    if !failures.is_empty() {
        println!();
        println!("=== Failures ===");
        for f in &failures {
            println!("  {f}");
        }
        panic!(
            "{} test(s) failed out of {} (see above for details)",
            failures.len(),
            entries.len()
        );
    }
}
