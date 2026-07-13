/// UTF-8 validation for JSON string content.
///
/// Provides exec validation of one UTF-8 character, proven to match
/// vstd's `valid_first_scalar` spec (RFC 3629 / Unicode Standard Table 3-7).
use vstd::prelude::*;
use vstd::utf8::*;

verus! {

// =============================================================================
// UTF-8 encoding boundaries
//
// Derived from RFC 3629 §4 "Syntax of UTF-8 Byte Sequences":
//
//   UTF8-1      = %x00-7F
//   UTF8-2      = %xC2-DF UTF8-tail
//   UTF8-3      = %xE0 %xA0-BF UTF8-tail / %xE1-EC 2(UTF8-tail) /
//                 %xED %x80-9F UTF8-tail / %xEE-EF 2(UTF8-tail)
//   UTF8-4      = %xF0 %x90-BF 2(UTF8-tail) / %xF1-F3 3(UTF8-tail) /
//                 %xF4 %x80-8F 2(UTF8-tail)
//   UTF8-tail   = %x80-BF
// =============================================================================

/// RFC 3629 §4 UTF8-1: single-byte characters end at 0x7F.
const ASCII_MAX: u8 = 0x7F;

/// RFC 3629 §4 UTF8-2: 2-byte leading bytes range [C2-DF].
/// (C0-C1 are excluded because they would produce overlong encodings
/// of U+0000..U+007F, per RFC 3629 §3.)
const UTF8_2BYTE_MIN: u8 = 0xC2;
const UTF8_2BYTE_MAX: u8 = 0xDF;

/// RFC 3629 §4 UTF8-3: 3-byte leading bytes range [E0-EF].
const UTF8_3BYTE_MIN: u8 = 0xE0;
const UTF8_3BYTE_MAX: u8 = 0xEF;

/// RFC 3629 §4 UTF8-4: 4-byte leading bytes range [F0-F4].
/// (F5-F7 are excluded because they would produce code points > U+10FFFF,
/// per RFC 3629 §3.)
const UTF8_4BYTE_MIN: u8 = 0xF0;
const UTF8_4BYTE_MAX: u8 = 0xF4;

/// RFC 3629 §4 UTF8-tail: continuation bytes range [80-BF].
const CONT_MIN: u8 = 0x80;
const CONT_MAX: u8 = 0xBF;

/// RFC 3629 §4 UTF8-3: when leading byte is E0, the second byte must be
/// in [A0-BF] to prevent overlong encoding of U+0000..U+07FF.
const E0_CONT_MIN: u8 = 0xA0;

/// RFC 3629 §4 UTF8-3: when leading byte is ED, the second byte must be
/// in [80-9F] to prevent encoding of surrogates U+D800..U+DFFF
/// (prohibited per RFC 3629 §3).
const ED_CONT_MAX: u8 = 0x9F;

/// RFC 3629 §4 UTF8-4: when leading byte is F0, the second byte must be
/// in [90-BF] to prevent overlong encoding of U+0000..U+FFFF.
const F0_CONT_MIN: u8 = 0x90;

/// RFC 3629 §4 UTF8-4: when leading byte is F4, the second byte must be
/// in [80-8F] to prevent encoding of code points > U+10FFFF.
const F4_CONT_MAX: u8 = 0x8F;

// =============================================================================
// Bit-vector helper lemmas
//
// These prove properties about decoded code points using pure integer
// arithmetic. The `by (bit_vector)` blocks require raw hex literals —
// named constants cannot be used inside them.
// =============================================================================

/// RFC 3629 §4 UTF8-2: a valid 2-byte sequence [C2-DF][80-BF] decodes to
/// U+0080..U+07FF, which is not overlong (genuinely needs 2 bytes).
proof fn lemma_2byte_not_overlong(b0: u8, b1: u8)
    requires 0xC2 <= b0 && b0 <= 0xDF, 0x80 <= b1 && b1 <= 0xBF,
    ensures not_overlong_encoding(codepoint_width_2(b0, b1), 2),
{
    assert(not_overlong_encoding(codepoint_width_2(b0, b1), 2)) by (bit_vector)
        requires 0xC2u8 <= b0 && b0 <= 0xDFu8, 0x80u8 <= b1 && b1 <= 0xBFu8;
}

/// RFC 3629 §4 UTF8-2: a valid 2-byte sequence [C2-DF][80-BF] decodes to
/// U+0080..U+07FF, well below the surrogate range U+D800..U+DFFF.
proof fn lemma_2byte_not_surrogate(b0: u8, b1: u8)
    requires 0xC2 <= b0 && b0 <= 0xDF, 0x80 <= b1 && b1 <= 0xBF,
    ensures not_surrogate(codepoint_width_2(b0, b1)),
{
    assert(not_surrogate(codepoint_width_2(b0, b1))) by (bit_vector)
        requires 0xC2u8 <= b0 && b0 <= 0xDFu8, 0x80u8 <= b1 && b1 <= 0xBFu8;
}

/// RFC 3629 §4 UTF8-3: a valid 3-byte sequence with the overlong guard
/// (E0 implies b1 >= A0) decodes to a code point >= U+0800.
proof fn lemma_3byte_not_overlong(b0: u8, b1: u8, b2: u8)
    requires
        0xE0 <= b0 && b0 <= 0xEF,
        0x80 <= b1 && b1 <= 0xBF,
        0x80 <= b2 && b2 <= 0xBF,
        (b0 != 0xE0 || b1 >= 0xA0),
    ensures not_overlong_encoding(codepoint_width_3(b0, b1, b2), 3),
{
    assert(not_overlong_encoding(codepoint_width_3(b0, b1, b2), 3)) by (bit_vector)
        requires 0xE0u8 <= b0 && b0 <= 0xEFu8,
                 0x80u8 <= b1 && b1 <= 0xBFu8,
                 0x80u8 <= b2 && b2 <= 0xBFu8,
                 (b0 != 0xE0u8 || b1 >= 0xA0u8);
}

/// RFC 3629 §3/§4 UTF8-3: a valid 3-byte sequence with the surrogate guard
/// (ED implies b1 <= 9F) decodes to a code point outside U+D800..U+DFFF.
/// RFC 3629 §3 prohibits encoding surrogates in UTF-8.
proof fn lemma_3byte_not_surrogate(b0: u8, b1: u8, b2: u8)
    requires
        0xE0 <= b0 && b0 <= 0xEF,
        0x80 <= b1 && b1 <= 0xBF,
        0x80 <= b2 && b2 <= 0xBF,
        (b0 != 0xED || b1 <= 0x9F),
    ensures not_surrogate(codepoint_width_3(b0, b1, b2)),
{
    assert(not_surrogate(codepoint_width_3(b0, b1, b2))) by (bit_vector)
        requires 0xE0u8 <= b0 && b0 <= 0xEFu8,
                 0x80u8 <= b1 && b1 <= 0xBFu8,
                 0x80u8 <= b2 && b2 <= 0xBFu8,
                 (b0 != 0xEDu8 || b1 <= 0x9Fu8);
}

/// RFC 3629 §4 UTF8-3: [E0][80-9F][80-BF] violates the overlong prohibition.
/// Decodes to U+0000..U+07FF which must use a shorter encoding (§3).
proof fn lemma_3byte_overlong(b0: u8, b1: u8, b2: u8)
    requires b0 == 0xE0, 0x80 <= b1 && b1 < 0xA0, 0x80 <= b2 && b2 <= 0xBF,
    ensures !not_overlong_encoding(codepoint_width_3(b0, b1, b2), 3),
{
    assert(!not_overlong_encoding(codepoint_width_3(b0, b1, b2), 3)) by (bit_vector)
        requires b0 == 0xE0u8, 0x80u8 <= b1 && b1 < 0xA0u8, 0x80u8 <= b2 && b2 <= 0xBFu8;
}

/// RFC 3629 §3: [ED][A0-BF][80-BF] encodes a surrogate code point
/// (U+D800..U+DFFF), which is explicitly prohibited.
proof fn lemma_3byte_surrogate(b0: u8, b1: u8, b2: u8)
    requires b0 == 0xED, 0xA0 <= b1 && b1 <= 0xBF, 0x80 <= b2 && b2 <= 0xBF,
    ensures !not_surrogate(codepoint_width_3(b0, b1, b2)),
{
    assert(!not_surrogate(codepoint_width_3(b0, b1, b2))) by (bit_vector)
        requires b0 == 0xEDu8, 0xA0u8 <= b1 && b1 <= 0xBFu8, 0x80u8 <= b2 && b2 <= 0xBFu8;
}

/// RFC 3629 §4 UTF8-4: a valid 4-byte sequence with both guards
/// (F0 implies b1 >= 90; F4 implies b1 <= 8F) decodes to U+10000..U+10FFFF,
/// which is not overlong.
proof fn lemma_4byte_not_overlong(b0: u8, b1: u8, b2: u8, b3: u8)
    requires
        0xF0 <= b0 && b0 <= 0xF4,
        0x80 <= b1 && b1 <= 0xBF,
        0x80 <= b2 && b2 <= 0xBF,
        0x80 <= b3 && b3 <= 0xBF,
        (b0 != 0xF0 || b1 >= 0x90),
        (b0 != 0xF4 || b1 <= 0x8F),
    ensures not_overlong_encoding(codepoint_width_4(b0, b1, b2, b3), 4),
{
    assert(not_overlong_encoding(codepoint_width_4(b0, b1, b2, b3), 4)) by (bit_vector)
        requires 0xF0u8 <= b0 && b0 <= 0xF4u8,
                 0x80u8 <= b1 && b1 <= 0xBFu8,
                 0x80u8 <= b2 && b2 <= 0xBFu8,
                 0x80u8 <= b3 && b3 <= 0xBFu8,
                 (b0 != 0xF0u8 || b1 >= 0x90u8),
                 (b0 != 0xF4u8 || b1 <= 0x8Fu8);
}

/// RFC 3629 §4 UTF8-4: a valid 4-byte sequence decodes to
/// U+10000..U+10FFFF, all above the surrogate range U+D800..U+DFFF.
proof fn lemma_4byte_not_surrogate(b0: u8, b1: u8, b2: u8, b3: u8)
    requires
        0xF0 <= b0 && b0 <= 0xF4,
        0x80 <= b1 && b1 <= 0xBF,
        0x80 <= b2 && b2 <= 0xBF,
        0x80 <= b3 && b3 <= 0xBF,
        (b0 != 0xF0 || b1 >= 0x90),
        (b0 != 0xF4 || b1 <= 0x8F),
    ensures not_surrogate(codepoint_width_4(b0, b1, b2, b3)),
{
    assert(not_surrogate(codepoint_width_4(b0, b1, b2, b3))) by (bit_vector)
        requires 0xF0u8 <= b0 && b0 <= 0xF4u8,
                 0x80u8 <= b1 && b1 <= 0xBFu8,
                 0x80u8 <= b2 && b2 <= 0xBFu8,
                 0x80u8 <= b3 && b3 <= 0xBFu8,
                 (b0 != 0xF0u8 || b1 >= 0x90u8),
                 (b0 != 0xF4u8 || b1 <= 0x8Fu8);
}

/// RFC 3629 §4 UTF8-4: [F0][80-8F][80-BF][80-BF] violates the overlong
/// prohibition — decodes to U+0000..U+FFFF which must use a shorter encoding.
proof fn lemma_4byte_overlong_f0(b0: u8, b1: u8, b2: u8, b3: u8)
    requires b0 == 0xF0, 0x80 <= b1 && b1 < 0x90, 0x80 <= b2 && b2 <= 0xBF, 0x80 <= b3 && b3 <= 0xBF,
    ensures !not_overlong_encoding(codepoint_width_4(b0, b1, b2, b3), 4),
{
    assert(!not_overlong_encoding(codepoint_width_4(b0, b1, b2, b3), 4)) by (bit_vector)
        requires b0 == 0xF0u8, 0x80u8 <= b1 && b1 < 0x90u8, 0x80u8 <= b2 && b2 <= 0xBFu8, 0x80u8 <= b3 && b3 <= 0xBFu8;
}

/// RFC 3629 §3: [F4][90-BF][80-BF][80-BF] decodes to a code point > U+10FFFF,
/// exceeding the maximum defined by RFC 3629 §3 (U+0000..U+10FFFF).
proof fn lemma_4byte_over_max_f4(b0: u8, b1: u8, b2: u8, b3: u8)
    requires b0 == 0xF4, 0x90 <= b1 && b1 <= 0xBF, 0x80 <= b2 && b2 <= 0xBF, 0x80 <= b3 && b3 <= 0xBF,
    ensures !not_overlong_encoding(codepoint_width_4(b0, b1, b2, b3), 4),
{
    assert(!not_overlong_encoding(codepoint_width_4(b0, b1, b2, b3), 4)) by (bit_vector)
        requires b0 == 0xF4u8, 0x90u8 <= b1 && b1 <= 0xBFu8, 0x80u8 <= b2 && b2 <= 0xBFu8, 0x80u8 <= b3 && b3 <= 0xBFu8;
}

/// RFC 3629 §3/§4: [C0-C1][80-BF] is overlong — decodes to U+0000..U+007F
/// which must use a 1-byte encoding. This is why UTF8-2 starts at C2.
proof fn lemma_c0c1_overlong(b0: u8, b1: u8)
    requires 0xC0 <= b0 && b0 <= 0xC1, 0x80 <= b1 && b1 <= 0xBF,
    ensures !not_overlong_encoding(codepoint_width_2(b0, b1), 2),
{
    assert(!not_overlong_encoding(codepoint_width_2(b0, b1), 2)) by (bit_vector)
        requires 0xC0u8 <= b0 && b0 <= 0xC1u8, 0x80u8 <= b1 && b1 <= 0xBFu8;
}

/// RFC 3629 §3: [F5-F7][80-BF][80-BF][80-BF] decodes to a code point
/// > U+10FFFF. Leading bytes F5-FF are not valid UTF-8 per RFC 3629 §4.
proof fn lemma_f5f7_over_max(b0: u8, b1: u8, b2: u8, b3: u8)
    requires 0xF5 <= b0 && b0 <= 0xF7, 0x80 <= b1 && b1 <= 0xBF, 0x80 <= b2 && b2 <= 0xBF, 0x80 <= b3 && b3 <= 0xBF,
    ensures !not_overlong_encoding(codepoint_width_4(b0, b1, b2, b3), 4),
{
    assert(!not_overlong_encoding(codepoint_width_4(b0, b1, b2, b3), 4)) by (bit_vector)
        requires 0xF5u8 <= b0 && b0 <= 0xF7u8, 0x80u8 <= b1 && b1 <= 0xBFu8, 0x80u8 <= b2 && b2 <= 0xBFu8, 0x80u8 <= b3 && b3 <= 0xBFu8;
}

// =============================================================================
// Main validation function
// =============================================================================

/// Result of validating one UTF-8 character.
pub(crate) enum Utf8CharResult {
    /// Valid character consuming `len` bytes (1-4).
    Ok { len: usize },
    /// Invalid byte sequence.
    Err,
}

/// Validate one UTF-8 character starting at `pos` in `input[pos..end)`.
///
/// Implements the ABNF grammar from RFC 3629 §4 "Syntax of UTF-8 Byte Sequences":
///
/// | Bytes | First   | Second  | Third   | Fourth  | Code points       |
/// |-------|---------|---------|---------|---------|-------------------|
/// | 1     | 00-7F   |         |         |         | U+0000..U+007F    |
/// | 2     | C2-DF   | 80-BF   |         |         | U+0080..U+07FF    |
/// | 3     | E0      | A0-BF   | 80-BF   |         | U+0800..U+0FFF    |
/// | 3     | E1-EC   | 80-BF   | 80-BF   |         | U+1000..U+CFFF    |
/// | 3     | ED      | 80-9F   | 80-BF   |         | U+D000..U+D7FF    |
/// | 3     | EE-EF   | 80-BF   | 80-BF   |         | U+E000..U+FFFF    |
/// | 4     | F0      | 90-BF   | 80-BF   | 80-BF   | U+10000..U+3FFFF  |
/// | 4     | F1-F3   | 80-BF   | 80-BF   | 80-BF   | U+40000..U+FFFFF  |
/// | 4     | F4      | 80-8F   | 80-BF   | 80-BF   | U+100000..U+10FFFF|
///
/// On success, returns the number of bytes consumed (1-4).
/// Proven: `Ok` iff vstd's `valid_first_scalar` holds on the subrange.
pub(crate) fn validate_utf8_char(input: &[u8], pos: usize, end: usize) -> (result: Utf8CharResult)
    requires
        pos < end,
        end <= input@.len(),
    ensures
        match result {
            Utf8CharResult::Ok { len } => {
                &&& 1 <= len <= 4
                &&& pos + len <= end
                &&& valid_first_scalar(input@.subrange(pos as int, end as int))
                &&& length_of_first_scalar(input@.subrange(pos as int, end as int)) == len as int
            },
            Utf8CharResult::Err => {
                !valid_first_scalar(input@.subrange(pos as int, end as int))
            },
        },
{
    let b0 = input[pos];
    let ghost sub = input@.subrange(pos as int, end as int);

    // 1-byte: [00-7F]
    if b0 <= 0x7F {
        proof {
            assert(is_leading_byte_width_1(sub[0]));
            assert(valid_leading_and_continuation_bytes_first_codepoint(sub));
            assert(decode_first_codepoint(sub) == codepoint_width_1(sub[0]));
            assert(not_overlong_encoding(decode_first_codepoint(sub), 1));
            assert(not_surrogate(decode_first_codepoint(sub)));
        }
        return Utf8CharResult::Ok { len: 1 };
    }

    // 2-byte: [C2-DF][80-BF]
    if UTF8_2BYTE_MIN <= b0 && b0 <= UTF8_2BYTE_MAX {
        if end - pos < 2 {
            proof {
                assert(!is_leading_byte_width_1(sub[0]));
                assert(sub.len() < 2);
                assert(!valid_leading_and_continuation_bytes_first_codepoint(sub));
                assert(!valid_first_scalar(sub));
            }
            return Utf8CharResult::Err;
        }
        let b1 = input[pos + 1];
        if !(CONT_MIN <= b1 && b1 <= CONT_MAX) {
            proof {
                assert(!is_leading_byte_width_1(sub[0]));
                assert(is_leading_byte_width_2(sub[0]));
                assert(!is_continuation_byte(sub[1]));
                assert(!valid_leading_and_continuation_bytes_first_codepoint(sub));
                assert(!valid_first_scalar(sub));
            }
            return Utf8CharResult::Err;
        }
        proof {
            assert(is_leading_byte_width_2(sub[0]));
            assert(is_continuation_byte(sub[1]));
            assert(valid_leading_and_continuation_bytes_first_codepoint(sub));
            assert(decode_first_codepoint(sub) == codepoint_width_2(sub[0], sub[1]));
            lemma_2byte_not_overlong(sub[0], sub[1]);
            lemma_2byte_not_surrogate(sub[0], sub[1]);
        }
        return Utf8CharResult::Ok { len: 2 };
    }

    // 3-byte: [E0-EF][80-BF][80-BF]
    if UTF8_3BYTE_MIN <= b0 && b0 <= UTF8_3BYTE_MAX {
        if end - pos < 3 {
            proof {
                assert(!is_leading_byte_width_1(sub[0]));
                assert(!is_leading_byte_width_2(sub[0]));
                assert(is_leading_byte_width_3(sub[0]));
                assert(sub.len() < 3);
                assert(!valid_leading_and_continuation_bytes_first_codepoint(sub));
                assert(!valid_first_scalar(sub));
            }
            return Utf8CharResult::Err;
        }
        let b1 = input[pos + 1];
        let b2 = input[pos + 2];
        if !(CONT_MIN <= b1 && b1 <= CONT_MAX) || !(CONT_MIN <= b2 && b2 <= CONT_MAX) {
            proof {
                assert(!is_leading_byte_width_1(sub[0]));
                assert(!is_leading_byte_width_2(sub[0]));
                assert(is_leading_byte_width_3(sub[0]));
                assert(!is_continuation_byte(sub[1]) || !is_continuation_byte(sub[2]));
                assert(!valid_leading_and_continuation_bytes_first_codepoint(sub));
                assert(!valid_first_scalar(sub));
            }
            return Utf8CharResult::Err;
        }
        if b0 == UTF8_3BYTE_MIN && b1 < E0_CONT_MIN {
            proof {
                assert(valid_leading_and_continuation_bytes_first_codepoint(sub));
                assert(decode_first_codepoint(sub) == codepoint_width_3(sub[0], sub[1], sub[2]));
                lemma_3byte_overlong(sub[0], sub[1], sub[2]);
                assert(!valid_first_scalar(sub));
            }
            return Utf8CharResult::Err;
        }
        if b0 == 0xED && b1 > ED_CONT_MAX {
            proof {
                assert(valid_leading_and_continuation_bytes_first_codepoint(sub));
                assert(decode_first_codepoint(sub) == codepoint_width_3(sub[0], sub[1], sub[2]));
                lemma_3byte_surrogate(sub[0], sub[1], sub[2]);
                assert(!valid_first_scalar(sub));
            }
            return Utf8CharResult::Err;
        }
        proof {
            assert(is_leading_byte_width_3(sub[0]));
            assert(is_continuation_byte(sub[1]));
            assert(is_continuation_byte(sub[2]));
            assert(valid_leading_and_continuation_bytes_first_codepoint(sub));
            assert(decode_first_codepoint(sub) == codepoint_width_3(sub[0], sub[1], sub[2]));
            lemma_3byte_not_overlong(sub[0], sub[1], sub[2]);
            lemma_3byte_not_surrogate(sub[0], sub[1], sub[2]);
        }
        return Utf8CharResult::Ok { len: 3 };
    }

    // 4-byte: [F0-F4][80-BF][80-BF][80-BF]
    if UTF8_4BYTE_MIN <= b0 && b0 <= UTF8_4BYTE_MAX {
        if end - pos < 4 {
            proof {
                assert(!is_leading_byte_width_1(sub[0]));
                assert(!is_leading_byte_width_2(sub[0]));
                assert(!is_leading_byte_width_3(sub[0]));
                assert(is_leading_byte_width_4(sub[0]));
                assert(sub.len() < 4);
                assert(!valid_leading_and_continuation_bytes_first_codepoint(sub));
                assert(!valid_first_scalar(sub));
            }
            return Utf8CharResult::Err;
        }
        let b1 = input[pos + 1];
        let b2 = input[pos + 2];
        let b3 = input[pos + 3];
        if !(CONT_MIN <= b1 && b1 <= CONT_MAX) || !(CONT_MIN <= b2 && b2 <= CONT_MAX) || !(CONT_MIN <= b3 && b3 <= CONT_MAX) {
            proof {
                assert(!is_leading_byte_width_1(sub[0]));
                assert(!is_leading_byte_width_2(sub[0]));
                assert(!is_leading_byte_width_3(sub[0]));
                assert(is_leading_byte_width_4(sub[0]));
                assert(!is_continuation_byte(sub[1]) || !is_continuation_byte(sub[2]) || !is_continuation_byte(sub[3]));
                assert(!valid_leading_and_continuation_bytes_first_codepoint(sub));
                assert(!valid_first_scalar(sub));
            }
            return Utf8CharResult::Err;
        }
        if b0 == UTF8_4BYTE_MIN && b1 < F0_CONT_MIN {
            proof {
                assert(valid_leading_and_continuation_bytes_first_codepoint(sub));
                assert(decode_first_codepoint(sub) == codepoint_width_4(sub[0], sub[1], sub[2], sub[3]));
                lemma_4byte_overlong_f0(sub[0], sub[1], sub[2], sub[3]);
                assert(!valid_first_scalar(sub));
            }
            return Utf8CharResult::Err;
        }
        if b0 == UTF8_4BYTE_MAX && b1 > F4_CONT_MAX {
            proof {
                assert(valid_leading_and_continuation_bytes_first_codepoint(sub));
                assert(decode_first_codepoint(sub) == codepoint_width_4(sub[0], sub[1], sub[2], sub[3]));
                lemma_4byte_over_max_f4(sub[0], sub[1], sub[2], sub[3]);
                assert(!valid_first_scalar(sub));
            }
            return Utf8CharResult::Err;
        }
        proof {
            assert(is_leading_byte_width_4(sub[0]));
            assert(is_continuation_byte(sub[1]));
            assert(is_continuation_byte(sub[2]));
            assert(is_continuation_byte(sub[3]));
            assert(valid_leading_and_continuation_bytes_first_codepoint(sub));
            assert(decode_first_codepoint(sub) == codepoint_width_4(sub[0], sub[1], sub[2], sub[3]));
            lemma_4byte_not_overlong(sub[0], sub[1], sub[2], sub[3]);
            lemma_4byte_not_surrogate(sub[0], sub[1], sub[2], sub[3]);
        }
        return Utf8CharResult::Ok { len: 4 };
    }

    // Invalid leading byte: 0x80-0xBF, 0xC0-0xC1, 0xF5-0xFF
    proof {
        assert(!is_leading_byte_width_1(sub[0]));
        // is_leading_byte_width_3 requires 0xE0-0xEF — not possible here
        assert(!is_leading_byte_width_3(sub[0]));

        if is_leading_byte_width_2(sub[0]) {
            // b0 must be 0xC0 or 0xC1 (since C2-DF handled above)
            if sub.len() >= 2 && is_continuation_byte(sub[1]) {
                lemma_c0c1_overlong(sub[0], sub[1]);
            }
        }
        if is_leading_byte_width_4(sub[0]) {
            // b0 must be 0xF5-0xF7 (since F0-F4 handled above)
            if sub.len() >= 4 && is_continuation_byte(sub[1])
                && is_continuation_byte(sub[2]) && is_continuation_byte(sub[3])
            {
                lemma_f5f7_over_max(sub[0], sub[1], sub[2], sub[3]);
            }
        }
    }
    Utf8CharResult::Err
}

} // verus!
