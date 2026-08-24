//! RFC 8259 conformance declarations.
//!
//! This module is a documentation-only home for RFC 8259 requirements that are
//! *not* implemented by this parser. Some of those are motivated choices
//! (.e.g not having any extensions) and others are out-of-scope requirements regarding
//! generation of JSON text.

// The parser accepts strictly RFC 8259 JSON and does not recognize any
// non-JSON forms or extensions.
//
//= https://www.rfc-editor.org/rfc/rfc8259#section-9
//= type=exception
//= reason=This parser accepts strictly RFC 8259 JSON only and does not accept non-JSON forms or extensions.
//# A JSON parser MAY accept non-JSON forms or extensions.
const EXTENSIONS_NOT_ACCEPTED: () = ();

// Generator-side requirement. This crate is a parser and never emits JSON
// text, so it cannot add a byte order mark.
//
//= https://www.rfc-editor.org/rfc/rfc8259#section-8.1
//= type=exception
//= reason=Generator-side requirement. This crate is a parser and never emits JSON text, so it cannot add a byte order mark.
//# Implementations MUST NOT add a byte order mark (U+FEFF) to the
//# beginning of a networked-transmitted JSON text.
const BOM_NOT_EMITTED: () = ();

// Optional (MAY). This parser does not skip a leading BOM; a BOM byte is an
// unrecognized token and causes a tokenization error.
//
//= https://www.rfc-editor.org/rfc/rfc8259#section-8.1
//= type=exception
//= reason=Optional (MAY). This parser does not skip a leading BOM; a BOM byte is an unrecognized token and causes a tokenization error.
//# In the interests of
//# interoperability, implementations that parse JSON texts MAY ignore
//# the presence of a byte order mark rather than treating it as an
//# error.
const BOM_NOT_IGNORED: () = ();

// Generator-side requirement. This crate is a parser, not a generator, and
// produces no JSON text.
//
//= https://www.rfc-editor.org/rfc/rfc8259#section-10
//= type=exception
//= reason=Generator-side requirement. This crate is a parser, not a generator, and produces no JSON text.
//# The resulting text MUST
//# strictly conform to the JSON grammar.
const NO_GENERATOR: () = ();
