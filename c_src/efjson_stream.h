/*
efjson_stream
======
define `EFJSON_STREAM_IMPL` to implement the library

# Requirements
  - C89/C++98
  - `CHAR_BIT` should be 8

# License
  The MIT License (MIT)

  Copyright (C) 2025 Jin Cai

  Permission is hereby granted, free of charge, to any person obtaining a copy
  of this software and associated documentation files (the "Software"), to deal
  in the Software without restriction, including without limitation the rights
  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
  copies of the Software, and to permit persons to whom the Software is
  furnished to do so, subject to the following conditions:

  The above copyright notice and this permission notice shall be included in all
  copies or substantial portions of the Software.

  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
  SOFTWARE.

# Example
```c
#include <stdio.h>
#define EFJSON_STREAM_IMPL
#include "efjson_stream.h"
static const char src[] =
  "{\
\"null\":null,\"true\":true,\"false\":false,\
\"string\":\"string,\\\"escape\\\",\\uD83D\\uDE00\",\
\"integer\":12,\"negative\":-12,\"fraction\":12.34,\"exponent\":1.234e2,\
\"array\":[\"1st element\",{\"object\":\"nesting\"}],\
\"object\":{\"1st\":[],\"2st\":{}}\
}";
int main() {
  efjsonStreamParser parser;
  unsigned i, n = sizeof(src) / sizeof(src[0]);
  efjsonStreamParser_init(&parser, 0);
  for(i = 0; i < n; ++i) {
    efjsonToken token = efjsonStreamParser_feedOne(&parser, src[i]);
    if(token.type == efjsonType_ERROR) {
      printf("%s\n", efjson_stringifyError(token.extra));
      return 1;
    } else {
      printf(
        "%-8s %-30s %u%c\n", efjson_stringifyLocation(token.location), efjson_stringifyType(token.type), token.index,
        token.done ? '*' : ' '
      );
    }
  }
  return 0;
}
```
*/
#ifndef EFJSON_STREAM_H
#define EFJSON_STREAM_H
#include <limits.h>
#include <stddef.h>


#if UCHAR_MAX != 0xFF
  #error "efjson.h: a byte must be 8 bits"
#endif
#if USHRT_MAX != 0xFFFF
  #error "efjson.h: 16-bit integer not exists"
#endif
typedef unsigned char efjsonUint8;
typedef unsigned short efjsonUint16;

#if UINT_MAX == 0xFFFFFFFF
typedef unsigned efjsonUint32;
#elif ULONG_MAX == 0xFFFFFFFF
typedef unsigned long efjsonUint32;
#else
  #error "efjson.h: 32-bit integer not exists"
#endif

typedef size_t efjsonPosition;
typedef unsigned efjsonStackLength;


#ifdef __cplusplus
  #define EFJSON_CODE_BEGIN extern "C" {
  #define EFJSON_CODE_END }
#else
  #define EFJSON_CODE_BEGIN
  #define EFJSON_CODE_END
#endif
EFJSON_CODE_BEGIN

/**
 * @def ul_likely
 * @brief Hints to the compiler that the condition is more likely to be true.
 */
/**
 * @def ul_unlikely
 * @brief Hints to the compiler that the condition is more likely to be false.
 */
#if defined(__has_builtin) && !defined(UL_PEDANTIC)
  #if __has_builtin(__builtin_expect)
    #ifndef ul_likely
      #define ul_likely(x) __builtin_expect(!!(x), 1)
    #endif
    #ifndef ul_unlikely
      #define ul_unlikely(x) __builtin_expect(!!(x), 0)
    #endif
  #endif
#endif /* ul_likely + ul_unlikely */
#ifndef ul_likely
  #define ul_likely(x) (x)
#endif /* ul_likely */
#ifndef ul_unlikely
  #define ul_unlikely(x) (x)
#endif /* ul_unlikely */


#ifndef EFJSON_PUBLIC
  #define EFJSON_PUBLIC
#endif
#ifndef EFJSON_PRIVATE
  #define EFJSON_PRIVATE static
#endif


/**
 * Configuration: Fixed stack size for `efjsonStreamParser`
 * If the value is >0, `efjsonStreamParser` will use a fixed-size stack;
 * otherwise, it will dynamically allocate the stack during parsing.
 */
#ifndef EFJSON_CONF_FIXED_STACK
  #define EFJSON_CONF_FIXED_STACK 64
#endif

#ifndef EFJSON_CONF_COMPRESS_STACK
  #define EFJSON_CONF_COMPRESS_STACK 1
#endif

/**
 * Configuration: Whether to enable UTF-8/UTF-16 encoder/decoder
 */
#ifndef EFJSON_CONF_UTF_ENCODER
  #define EFJSON_CONF_UTF_ENCODER 1
#endif

/**
 * Configuration: Whether to comply with the Unicode standard
 * If this value is nonzero, we will follow the Unicode requirements in JSON5.
 * Otherwise, the following changes will apply:
 * - Whitespace only includes [\r\t\n ] (ignores `efjsonOption_JSON5_WHITESPACE`u)
 * - Identifier start only accepts [$_[a-z][A-Z]], identifier continue only accepts [$_[a-z][A-Z][0-9]]
 * - Printable characters only accept [\x20-\x7F]
 */
#ifndef EFJSON_CONF_UNICODE
  #define EFJSON_CONF_UNICODE 1
#endif

/**
 * Configuration: Provide pretty string for enums
 */
#ifndef EFJSON_CONF_PRETTIER
  #define EFJSON_CONF_PRETTIER 1
#endif

/**
 * Configuration: Provide pretty string for `efjsonCategory`
 */
#ifndef EFJSON_CONF_PRETTIER_CATEGORY
  #define EFJSON_CONF_PRETTIER_CATEGORY EFJSON_CONF_PRETTIER
#endif
/**
 * Configuration: Provide pretty string for `efjsonError`
 */
#ifndef EFJSON_CONF_PRETTIER_ERROR
  #define EFJSON_CONF_PRETTIER_ERROR EFJSON_CONF_PRETTIER
#endif

/**
 * Configuration: Provide pretty string for `efjsonLocation`
 */
#ifndef EFJSON_CONF_PRETTIER_LOCATION
  #define EFJSON_CONF_PRETTIER_LOCATION EFJSON_CONF_PRETTIER
#endif

/**
 * Configuration: Provide pretty string for `efjsonType`
 */
#ifndef EFJSON_CONF_PRETTIER_TYPE
  #define EFJSON_CONF_PRETTIER_TYPE EFJSON_CONF_PRETTIER
#endif

/**
 * Configuration: Whether to check `efjsonPosition` overflow
 */
#ifndef EFJSON_CONF_CHECK_POSITION_OVERFLOW
  #define EFJSON_CONF_CHECK_POSITION_OVERFLOW 1
#endif

/**
 * Configuration: Whether to check `size_t` overflow
 * It's valid only when `EFJSON_CONF_FIXED_STACK` <= 0.
 */
#ifndef EFJSON_CONF_CHECK_SIZET_OVERFLOW
  #define EFJSON_CONF_CHECK_SIZET_OVERFLOW 0
#endif

/**
 * Configuration: Whether to expose some Unicode APIs
 */
#ifndef EFJSON_CONF_EXPOSE_UNICODE
  #define EFJSON_CONF_EXPOSE_UNICODE 1
#endif

/**
 * Configuration: If the escape sequence in a string contains a surrogate pair, whether to combine them
 * This configuration only affects `string`.
 */
#ifndef EFJSON_CONF_COMBINE_ESCAPED_SURROGATE
  #define EFJSON_CONF_COMBINE_ESCAPED_SURROGATE 1
#endif

/**
 * Configuration: Whether to check the input Unicode codepoint is valid
 * This configuration only affects `string` and `identifier`, as they may contain any character.
 */
#ifndef EFJSON_CONF_CHECK_INPUT_UTF
  #define EFJSON_CONF_CHECK_INPUT_UTF 1
#endif

/**
 * Configuration: Whether to check the escaped Unicode codepoint is valid
 * This configuration only affects `string` and `identifier`.
 */
#ifndef EFJSON_CONF_CHECK_ESCAPE_UTF
  #define EFJSON_CONF_CHECK_ESCAPE_UTF 1
#endif


#if EFJSON_CONF_EXPOSE_UNICODE
  #define EFJSON_UAPI EFJSON_PUBLIC
/* this function is not used by this library; it is only provided to help improve error messages */
EFJSON_UAPI int efjson_isGraph(efjsonUint32 u);
#else
  #define EFJSON_UAPI EFJSON_PRIVATE
#endif
EFJSON_UAPI int efjson_isWhitespace(efjsonUint32 u, int fitJson5);
EFJSON_UAPI int efjson_isIdentifierStart(efjsonUint32 u);
EFJSON_UAPI int efjson_isIdentifierNext(efjsonUint32 u);


#if EFJSON_CONF_UTF_ENCODER
typedef struct efjsonUtf8Decoder {
  efjsonUint32 code;
  efjsonUint16 rest, total;
} efjsonUtf8Decoder;
EFJSON_PUBLIC size_t efjsonUtf8Decoder_sizeof(void);
EFJSON_PUBLIC efjsonUtf8Decoder* efjsonUtf8Decoder_new(void);
EFJSON_PUBLIC void efjsonUtf8Decoder_destroy(efjsonUtf8Decoder* decoder);
EFJSON_PUBLIC void efjsonUtf8Decoder_init(efjsonUtf8Decoder* decoder);
/** return number of decoded bytes (0/1 for normal, -1 for error) */
EFJSON_PUBLIC int efjsonUtf8Decoder_feed(efjsonUtf8Decoder* decoder, efjsonUint32* result, efjsonUint8 c);
/** return number of encoded bytes (1-4 for normal, -1 for error) */
EFJSON_PUBLIC int efjson_EncodeUtf8(efjsonUint8* p, efjsonUint32 u);

typedef struct efjsonUtf16Decoder {
  efjsonUint16 first;
} efjsonUtf16Decoder;
EFJSON_PUBLIC size_t efjsonUtf16Decoder_sizeof(void);
EFJSON_PUBLIC efjsonUtf16Decoder* efjsonUtf16Decoder_new(void);
EFJSON_PUBLIC void efjsonUtf16Decoder_destroy(efjsonUtf16Decoder* decoder);
EFJSON_PUBLIC void efjsonUtf16Decoder_init(efjsonUtf16Decoder* decoder);
/** return number of decoded bytes (0/1 for normal, -1 for error) */
EFJSON_PUBLIC int efjsonUtf16Decoder_feed(efjsonUtf16Decoder* decoder, efjsonUint32* result, efjsonUint16 c);
/** return number of encoded bytes (1-2 for normal, -1 for error) */
EFJSON_PUBLIC int efjson_EncodeUtf16(efjsonUint16* p, efjsonUint32 u);
#endif


#define efjson_TOKEN_CATEGORY_SHIFT (4u)
enum efjsonCategory /* : efjsonUint8 */ {
  efjsonCategory_ERROR,
  efjsonCategory_WHITESPACE,
  efjsonCategory_EOF,
  efjsonCategory_NULL,
  efjsonCategory_BOOLEAN,
  efjsonCategory_STRING,
  efjsonCategory_NUMBER,
  efjsonCategory_OBJECT,
  efjsonCategory_ARRAY,
  efjsonCategory_IDENTIFIER,
  efjsonCategory_COMMENT
};
enum efjsonTokenType /* : efjsonUint16 */ {
  efjsonType_ERROR = 0,

  efjsonType_WHITESPACE = efjsonCategory_WHITESPACE << efjson_TOKEN_CATEGORY_SHIFT | 0x0,
  efjsonType_EOF = efjsonCategory_EOF << efjson_TOKEN_CATEGORY_SHIFT | 0x0,
  efjsonType_NULL = efjsonCategory_NULL << efjson_TOKEN_CATEGORY_SHIFT | 0x0,

  efjsonType_FALSE = efjsonCategory_BOOLEAN << efjson_TOKEN_CATEGORY_SHIFT | 0x0,
  efjsonType_TRUE = efjsonCategory_BOOLEAN << efjson_TOKEN_CATEGORY_SHIFT | 0x1,

  efjsonType_STRING_START = efjsonCategory_STRING << efjson_TOKEN_CATEGORY_SHIFT | 0x0,
  efjsonType_STRING_END = efjsonCategory_STRING << efjson_TOKEN_CATEGORY_SHIFT | 0x1,
  efjsonType_STRING_NORMAL = efjsonCategory_STRING << efjson_TOKEN_CATEGORY_SHIFT | 0x2,
  efjsonType_STRING_ESCAPE_START = efjsonCategory_STRING << efjson_TOKEN_CATEGORY_SHIFT | 0x3,
  efjsonType_STRING_ESCAPE = efjsonCategory_STRING << efjson_TOKEN_CATEGORY_SHIFT | 0x4,
  efjsonType_STRING_ESCAPE_UNICODE_START = efjsonCategory_STRING << efjson_TOKEN_CATEGORY_SHIFT | 0x5,
  efjsonType_STRING_ESCAPE_UNICODE = efjsonCategory_STRING << efjson_TOKEN_CATEGORY_SHIFT | 0x6,
  efjsonType_STRING_NEXT_LINE = efjsonCategory_STRING << efjson_TOKEN_CATEGORY_SHIFT | 0x7,
  efjsonType_STRING_ESCAPE_HEX_START = efjsonCategory_STRING << efjson_TOKEN_CATEGORY_SHIFT | 0x8,
  efjsonType_STRING_ESCAPE_HEX = efjsonCategory_STRING << efjson_TOKEN_CATEGORY_SHIFT | 0x9,

  efjsonType_NUMBER_INTEGER_DIGIT = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0x0,
  efjsonType_NUMBER_FRACTION_DIGIT = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0x1,
  efjsonType_NUMBER_EXPONENT_DIGIT = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0x2,
  efjsonType_NUMBER_INTEGER_SIGN = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0x3,
  efjsonType_NUMBER_EXPONENT_SIGN = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0x4,
  efjsonType_NUMBER_FRACTION_START = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0x5,
  efjsonType_NUMBER_EXPONENT_START = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0x6,
  efjsonType_NUMBER_NAN = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0x7,
  efjsonType_NUMBER_INFINITY = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0x8,
  efjsonType_NUMBER_HEX_START = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0x9,
  efjsonType_NUMBER_HEX = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0xA,
  efjsonType_NUMBER_OCT_START = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0xB,
  efjsonType_NUMBER_OCT = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0xC,
  efjsonType_NUMBER_BIN_START = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0xD,
  efjsonType_NUMBER_BIN = efjsonCategory_NUMBER << efjson_TOKEN_CATEGORY_SHIFT | 0xE,

  efjsonType_OBJECT_START = efjsonCategory_OBJECT << efjson_TOKEN_CATEGORY_SHIFT | 0x0,
  efjsonType_OBJECT_NEXT = efjsonCategory_OBJECT << efjson_TOKEN_CATEGORY_SHIFT | 0x1,
  efjsonType_OBJECT_VALUE_START = efjsonCategory_OBJECT << efjson_TOKEN_CATEGORY_SHIFT | 0x2,
  efjsonType_OBJECT_END = efjsonCategory_OBJECT << efjson_TOKEN_CATEGORY_SHIFT | 0x3,

  efjsonType_ARRAY_START = efjsonCategory_ARRAY << efjson_TOKEN_CATEGORY_SHIFT | 0x0,
  efjsonType_ARRAY_NEXT = efjsonCategory_ARRAY << efjson_TOKEN_CATEGORY_SHIFT | 0x1,
  efjsonType_ARRAY_END = efjsonCategory_ARRAY << efjson_TOKEN_CATEGORY_SHIFT | 0x2,

  efjsonType_IDENTIFIER_NORMAL = efjsonCategory_IDENTIFIER << efjson_TOKEN_CATEGORY_SHIFT | 0x0,
  efjsonType_IDENTIFIER_ESCAPE_START = efjsonCategory_IDENTIFIER << efjson_TOKEN_CATEGORY_SHIFT | 0x1,
  efjsonType_IDENTIFIER_ESCAPE = efjsonCategory_IDENTIFIER << efjson_TOKEN_CATEGORY_SHIFT | 0x2,

  efjsonType_COMMENT_MAY_START = efjsonCategory_COMMENT << efjson_TOKEN_CATEGORY_SHIFT | 0x0,
  efjsonType_COMMENT_SINGLE_LINE = efjsonCategory_COMMENT << efjson_TOKEN_CATEGORY_SHIFT | 0x1,
  efjsonType_COMMENT_MULTI_LINE = efjsonCategory_COMMENT << efjson_TOKEN_CATEGORY_SHIFT | 0x3,
  efjsonType_COMMENT_MULTI_LINE_END = efjsonCategory_COMMENT << efjson_TOKEN_CATEGORY_SHIFT | 0x4
};
enum efjsonLocation /* : efjsonUint8 */ {
  efjsonLocation_ROOT,
  efjsonLocation_KEY,
  efjsonLocation_VALUE,
  efjsonLocation_ELEMENT,
  efjsonLocation_ARRAY,
  efjsonLocation_OBJECT
};
enum efjsonError /* efjsonUint8 */ {
  efjsonError_NONE = 0,
  efjsonError_ALLOC_FAILED,
  efjsonError_TOO_MANY_RECURSIONS,
  efjsonError_POSITION_OVERFLOW,
  efjsonError_INVALID_INPUT_UTF,
  efjsonError_INVALID_ESCAPED_UTF,
  efjsonError_INCOMPLETE_SURROGATE_PAIR,
  /* << other >> */
  efjsonError_COMMENT_FORBIDDEN = 0x80,
  efjsonError_EOF,
  efjsonError_NONWHITESPACE_AFTER_END,
  efjsonError_CONTENT_AFTER_EOF,
  efjsonError_TRAILING_COMMA_FORBIDDEN,
  efjsonError_UNEXPECTED,
  efjsonError_WRONG_BRACKET,
  efjsonError_WRONG_COLON,
  /* << array >> */
  efjsonError_COMMA_IN_EMPTY_ARRAY,
  /* << object >> */
  efjsonError_BAD_IDENTIFIER_ESCAPE,
  efjsonError_BAD_PROPERTY_NAME_IN_OBJECT,
  efjsonError_COMMA_IN_EMPTY_OBJECT,
  efjsonError_EMPTY_VALUE_IN_OBJECT,
  efjsonError_EXPECTED_COLON,
  efjsonError_INVALID_IDENTIFIER,
  efjsonError_INVALID_IDENTIFIER_ESCAPE,
  efjsonError_REPEATED_COLON,
  /* << string >> */
  efjsonError_BAD_ESCAPE_IN_STRING,
  efjsonError_BAD_HEX_ESCAPE_IN_STRING,
  efjsonError_BAD_UNICODE_ESCAPE_IN_STRING,
  efjsonError_CONTROL_CHARACTER_FORBIDDEN_IN_STRING,
  efjsonError_SINGLE_QUOTE_FORBIDDEN,
  /* << number >> */
  efjsonError_EMPTY_EXPONENT_PART,
  efjsonError_EMPTY_FRACTION_PART,
  efjsonError_EMPTY_INTEGER_PART,
  efjsonError_EXPONENT_NOT_ALLOWED,
  efjsonError_FRACTION_NOT_ALLOWED,
  efjsonError_LEADING_ZERO_FORBIDDEN,
  efjsonError_POSITIVE_SIGN_FORBIDDEN,
  efjsonError_UNEXPECTED_IN_NUMBER
};

#if EFJSON_CONF_PRETTIER_CATEGORY
EFJSON_PUBLIC const char* efjson_stringifyCategory(efjsonUint8 category);
#endif
#if EFJSON_CONF_PRETTIER_TYPE
EFJSON_PUBLIC const char* efjson_stringifyType(efjsonUint16 type);
#endif
#if EFJSON_CONF_PRETTIER_LOCATION
EFJSON_PUBLIC const char* efjson_stringifyLocation(efjsonUint8 location);
#endif
#if EFJSON_CONF_PRETTIER_ERROR
EFJSON_PUBLIC const char* efjson_stringifyError(efjsonUint8 error);
#endif


typedef struct efjsonToken {
  /**
   * The type of the token. (see `efjsonTokenType`)
   *
   * When `type` is `efjsonType_ERROR`, the `extra` field represents the error code (see `efjsonError`).
   *
   * `index` and `done` are only valid for specific `type`, and the following table indicates their range.
   * Further, only when `done` is `1`, `extra` field is for the escaped codepoint.
   *
   * | `type`                                   | `index` | `done` |
   * | ---------------------------------------- | ------- | ------ |
   * | `efjsonType_NULL`                        | 0..3    | 0,1    |
   * | `efjsonType_FALSE`                       | 0..4    | 0,1    |
   * | `efjsonType_TRUE`                        | 0..3    | 0,1    |
   * | `efjsonType_NUMBER_INFINITY`             | 0..7    | 0,1    |
   * | `efjsonType_NUMBER_NAN`                  | 0..2    | 0,1    |
   * | `efjsonType_STRING_ESCAPE`               | 0       | 1      |
   * | `efjsonType_STRING_ESCAPE_UNICODE` [^1]  | 0..4    | 0,1    |
   * | `efjsonType_STRING_ESCAPE_HEX`           | 0,1     | 0,1    |
   * | `efjsonType_IDENTIFIER_ESCAPE_START`[^2] | 0,1     | 0,1    |
   * | `efjsonType_IDENTIFIER_ESCAPE`           | 0..3    | 0,1    |
   *
   * [^1]: If `EFJSON_CONF_COMBINE_ESCAPED_SURROGATE` is set to `1`, the range of `index` will be `0..9`.
   * [^2]: The `extra` field is always set to `0`.
   */
  efjsonUint8 type;
  /**
   * The location of the token in the JSON value. (see `efjsonLocation`)
   */
  efjsonUint8 location;
  /**
   * The index in the sequence.
   */
  efjsonUint8 index;
  /**
   * Whether the sequence is done.
   */
  efjsonUint8 done;
  /**
   * Extra info (error code / escaped codepoint).
   */
  efjsonUint32 extra;
} efjsonToken;
EFJSON_PUBLIC efjsonUint8 efjson_getError(efjsonToken token);


enum efjsonOption {
  /* << white space >> */
  /**
   * whether to accept whitespace in JSON5
   */
  efjsonOption_JSON5_WHITESPACE = 0x000001u,

  /* << array >> */
  /**
   * whether to accept a single trailing comma in array
   * @example '[1,]'
   */
  efjsonOption_TRAILING_COMMA_IN_ARRAY = 0x000002u,

  /* << object >> */
  /**
   * whether to accept a single trailing comma in object
   * @example '{"a":1,}'
   */
  efjsonOption_TRAILING_COMMA_IN_OBJECT = 0x000004u,
  /**
   * whether to accept identifier key in object
   * @example '{a:1}'
   */
  efjsonOption_IDENTIFIER_KEY = 0x000008u,

  /* << string >> */
  /**
   * whether to accept single quote in string
   * @example "'a'"
   */
  efjsonOption_SINGLE_QUOTE = 0x000010u,
  /**
   * whether to accept multi-line string
   * @example '"a\\\nb"'
   */
  efjsonOption_MULTILINE_STRING = 0x000020u,
  /**
   * whether to accept JSON5 string escape
   * @example '"\\x01"', '\\v', '\\0'
   */
  efjsonOption_JSON5_STRING_ESCAPE = 0x000040u,

  /* << number >> */
  /**
   * whether to accept positive sign in number
   * @example '+1', '+0'
   */
  efjsonOption_POSITIVE_SIGN = 0x000080u,
  /**
   * whether to accept empty fraction in number
   * @example '1.', '0.'
   */
  efjsonOption_EMPTY_FRACTION = 0x000100u,
  /**
   * whether to accept empty integer in number
   * @example '.1', '.0'
   */
  efjsonOption_EMPTY_INTEGER = 0x000200u,
  /**
   * whether to accept NaN
   */
  efjsonOption_NAN = 0x000400u,
  /**
   * whether to accept Infinity
   */
  efjsonOption_INFINITY = 0x000800u,
  /**
   * whether to accept hexadecimal integer
   * @example '0x1', '0x0'
   */
  efjsonOption_HEXADECIMAL_INTEGER = 0x001000u,
  /**
   * whether to accept octal integer
   * @example '0o1', '0o0'
   */
  efjsonOption_OCTAL_INTEGER = 0x002000u,
  /**
   * whether to accept binary integer
   * @example '0b1', '0b0'
   */
  efjsonOption_BINARY_INTEGER = 0x004000u,

  /* << comment >> */
  /**
   * whether to accept single line comment
   * @example '// a comment'
   */
  efjsonOption_SINGLE_LINE_COMMENT = 0x008000u,
  /**
   * whether to accept multi-line comment
   */
  efjsonOption_MULTI_LINE_COMMENT = 0x010000u
};

typedef struct efjsonStreamParser {
  efjsonPosition position, line, column;
  efjsonUint32 option;
  efjsonUint8 /* efjsonLoc__* */ location;
  efjsonUint8 /* efjsonVal__* */ state;
  efjsonUint8 flag;
  efjsonUint8 substate;
  efjsonUint16 escape;
#if EFJSON_CONF_COMBINE_ESCAPED_SURROGATE
  efjsonUint16 prevPair;
#endif

  efjsonStackLength len;
#if EFJSON_CONF_FIXED_STACK > 0
  efjsonUint8 stack[EFJSON_CONF_FIXED_STACK];
#else
  efjsonStackLength cap;
  efjsonUint8* stack;
#endif
} efjsonStreamParser;

EFJSON_PUBLIC size_t efjsonStreamParser_sizeof(void);
EFJSON_PUBLIC efjsonStreamParser* efjsonStreamParser_new(efjsonUint32 option);
EFJSON_PUBLIC void efjsonStreamParser_destroy(efjsonStreamParser* parser);
EFJSON_PUBLIC void efjsonStreamParser_init(efjsonStreamParser* parser, efjsonUint32 option);
EFJSON_PUBLIC void(efjsonStreamParser_deinit)(efjsonStreamParser* parser);
EFJSON_PUBLIC int(efjsonStreamParser_initCopy)(efjsonStreamParser* parser, const efjsonStreamParser* src);
EFJSON_PUBLIC void(efjsonStreamParser_initMove)(efjsonStreamParser* parser, efjsonStreamParser* src);
EFJSON_PUBLIC efjsonStreamParser* efjsonStreamParser_newCopy(const efjsonStreamParser* src);

EFJSON_PUBLIC efjsonToken efjsonStreamParser_feedOne(efjsonStreamParser* parser, efjsonUint32 u);
/**
 * Pass multiple UTF-32 codepoints to the parser.
 * @note If the string ends, remember to pass `EOF` to parser.
 * @return 0 if failed (and error will be writen to `dest[0]`), or the number of tokens if success.
 */
EFJSON_PUBLIC size_t
efjsonStreamParser_feed(efjsonStreamParser* parser, efjsonToken* dest, const efjsonUint32* src, size_t len);

EFJSON_PUBLIC efjsonPosition efjsonStreamParser_getLine(const efjsonStreamParser* parser);
EFJSON_PUBLIC efjsonPosition efjsonStreamParser_getColumn(const efjsonStreamParser* parser);
EFJSON_PUBLIC efjsonPosition efjsonStreamParser_getPosition(const efjsonStreamParser* parser);

enum efjsonStage {
  efjsonStage_NOT_STARTED = -1,
  efjsonStage_PARSING = 0,
  efjsonStage_ENDED = 1
};
EFJSON_PUBLIC enum efjsonStage efjsonStreamParser_getStage(const efjsonStreamParser* parser);
EFJSON_CODE_END


#if EFJSON_CONF_FIXED_STACK > 0
  #include <string.h>
  #define efjsonStreamParser_deinit(parser) ((void)(parser))
  #define efjsonStreamParser_initCopy(parser, src) ((void)memmove((parser), (src), sizeof(efjsonStreamParser)), 0)
  #define efjsonStreamParser_initMove(parser, src) ((void)memmove((parser), (src), sizeof(efjsonStreamParser)))
#endif


#ifdef EFJSON_STREAM_IMPL
  #include <assert.h>
  #include <string.h>
  #include <stdlib.h>

EFJSON_CODE_BEGIN
  #ifdef __cplusplus
    #define efjson_cast(T, v) (static_cast<T>(v))
    #define efjson_reptr(T, p) (reinterpret_cast<T>(p))
  #else
    #define efjson_cast(T, v) ((T)(v))
    #define efjson_reptr(T, p) ((T)(p))
  #endif

  #define efjson_umax(T) efjson_cast(T, ~efjson_cast(T, 0))

  /**
   * @def ul_fallthrough
   * @brief Marks a fallthrough in a switch statement (it's used to suppress warnings).
   */
  #if !defined(ul_fallthrough) && defined(__STDC_VERSION__) && __STDC_VERSION__ >= 202311L && defined(__has_c_attribute)
    #if __has_c_attribute(fallthrough)
      #define ul_fallthrough [[fallthrough]]
    #endif
  #endif /* ul_fallthrough */
  #if !defined(ul_fallthrough) && (defined(__cplusplus) && __cplusplus >= 201103L && defined(__has_cpp_attribute))
    #if __has_cpp_attribute(fallthrough)
      #define ul_fallthrough [[fallthrough]]
    #endif
  #endif /* ul_fallthrough */
  #if !defined(ul_fallthrough) && !defined(UL_PEDANTIC)
    #if defined(__has_attribute)
      #if __has_attribute(fallthrough)
        #define ul_fallthrough __attribute__((__fallthrough__))
      #endif
    #endif
  #endif /* ul_fallthrough */
  #ifndef ul_fallthrough
    #define ul_fallthrough ((void)0)
  #endif /* ul_fallthrough */

  #define efjson_assert(cond) assert(cond)
  #define efjson_condexpr(cond, expr) (efjson_assert(cond), (expr))

  #ifdef EFJSON_CONF_CHECK_SIZET_OVERFLOW
  #endif


  /******************************
   * Format
   ******************************/


  #if EFJSON_CONF_PRETTIER_CATEGORY
EFJSON_PRIVATE const char* const efjson__CATEGORY_FORMAT[] = { "<error>", "whitespace", "eof",    "null",
                                                               "boolean", "string",     "number", "object",
                                                               "array",   "identifier", "comment" };
EFJSON_PUBLIC const char* efjson_stringifyCategory(efjsonUint8 category) {
  return category <= efjsonCategory_COMMENT ? efjson__CATEGORY_FORMAT[category] : "<unknown>";
}
  #endif


  #if EFJSON_CONF_PRETTIER_TYPE
EFJSON_PRIVATE const char* const efjson__TYPE_FORMAT[] = {
  "<error>\0",
  "[whitespace]\0",
  "[eof]\0",
  "[null]\0",
  "[boolean]false\0[boolean]true\0",
  "[string]start\0[string]end\0[string]normal\0\
[string]escape_start\0[string]escape\0\
[string]escape_unicode_start\0[string]escape_unicode\0\
[string]next_line\0\
[string]escape_hex_start\0[string]escape_hex\0",
  "[number]integer_digit\0[number]fraction_digit\0[number]exponent_digit\0\
[number]integer_sign\0[number]exponent_sign\0\
[number]fraction_start\0[number]exponent_start\0\
[number]nan\0[number]infinity\0\
[number]hex_start\0[number]hex\0\
[number]oct_start\0[number]oct\0\
[number]bin_start\0[number]bin\0",
  "[object]start\0[object]next\0[object]value_start\0[object]end\0",
  "[array]start\0[array]next\0[array]end\0",
  "[identifier]normal\0[identifier]escape_start\0[identifier]escape\0",
  "[comment]may_start\0[comment]single_line\0[comment]multi_line\0[comment]multi_line_end\0",
};
EFJSON_PUBLIC const char* efjson_stringifyType(efjsonUint16 type) {
  efjsonUint16 category = type >> efjson_TOKEN_CATEGORY_SHIFT;
  if(category <= efjsonCategory_COMMENT) {
    const char* ptr = efjson__TYPE_FORMAT[category];
    type &= (1u << efjson_TOKEN_CATEGORY_SHIFT) - 1;
    while(*ptr != 0) {
      if(type == 0) return ptr;
      type--;
      ptr += strlen(ptr) + 1;
    }
  }
  return "<unknown>";
}
  #endif


  #if EFJSON_CONF_PRETTIER_LOCATION
EFJSON_PRIVATE const char* const efjson__LOCATION_FORMAT[] = {
  "root", "key", "value", "element", "array", "object",
};
EFJSON_PUBLIC const char* efjson_stringifyLocation(efjsonUint8 location) {
  return location <= efjsonLocation_OBJECT ? efjson__LOCATION_FORMAT[location] : "<unknown>";
}
  #endif


  #if EFJSON_CONF_PRETTIER_ERROR
EFJSON_PRIVATE const char* const efjson__ERROR_FORMAT1[] = {
  "<no error>",          "<allocation failed>",   "<too many recursions>",       "<position overflow>",
  "<invalid input UTF>", "<invalid escaped UTF>", "<incomplete surrogate pair>",
};
EFJSON_PRIVATE const char* const efjson__ERROR_FORMAT2[] = {
  /* << other >> */
  "comment not allowed",
  "structure broken because of EOF",
  "unexpected non-whitespace character after end of JSON",
  "content after EOF",
  "trailing comma not allowed",
  "unexpected character",
  "wrong bracket",
  "colon only allowed between property name and value",
  /* << array >> */
  "empty array with trailing comma not allowed",
  /* << object >> */
  "the escape sequence for an identifier must start with \"/u\"",
  "property name must be a string",
  "empty object with trailing comma not allowed",
  "unexpected empty value in object",
  "colon expected between property name and value",
  "invalid identifier in JSON string",
  "invalid identifier escape sequence in JSON5 identifier",
  "repeated colon not allowed",
  /* << string >> */
  "bad escape sequence in JSON string",
  "bad hex escape sequence in JSON string",
  "bad Unicode escape sequence in JSON string",
  "control character not allowed in JSON string",
  "single quote not allowed",
  /* << number >> */
  "the exponent part of a number cannot be empty",
  "the fraction part of a number cannot be empty",
  "the integer part of a number cannot be empty",
  "exponent part not allowed in non-decimal number",
  "fraction part not allowed in non-decimal number",
  "leading zero not allowed",
  "positive sign not allowed",
  "unexpected character in number",
};

EFJSON_PUBLIC const char* efjson_stringifyError(efjsonUint8 error) {
  if(error < 0x80) {
    if(error <= efjsonError_INCOMPLETE_SURROGATE_PAIR) return efjson__ERROR_FORMAT1[error];
  } else {
    if(error <= efjsonError_UNEXPECTED_IN_NUMBER) return efjson__ERROR_FORMAT2[error - 0x80];
  }
  return "<unkown>";
}
  #endif


  /******************************
   * Unicode validation
   ******************************/

  #if EFJSON_CONF_UNICODE
EFJSON_PRIVATE const efjsonUint16 EFJSON__EXTRA_WHITESPACE[] = { 0x000B, 0x000C, 0x00A0, 0xFEFF, 0x1680, 0x2000, 0x2001,
                                                                 0x2002, 0x2003, 0x2004, 0x2005, 0x2006, 0x2007, 0x2008,
                                                                 0x2009, 0x200A, 0x202F, 0x205F, 0x3000 };

EFJSON_PRIVATE const efjsonUint16 EFJSON__IDENTIFIER_START1[][2] = {
  { 0x0024, 0x0024 }, { 0x0041, 0x005A }, { 0x005F, 0x005F }, { 0x0061, 0x007A }, { 0x00AA, 0x00AA },
  { 0x00B5, 0x00B5 }, { 0x00BA, 0x00BA }, { 0x00C0, 0x00D6 }, { 0x00D8, 0x00F6 }, { 0x00F8, 0x02C1 },
  { 0x02C6, 0x02D1 }, { 0x02E0, 0x02E4 }, { 0x02EC, 0x02EC }, { 0x02EE, 0x02EE }, { 0x0370, 0x0374 },
  { 0x0376, 0x0377 }, { 0x037A, 0x037D }, { 0x037F, 0x037F }, { 0x0386, 0x0386 }, { 0x0388, 0x038A },
  { 0x038C, 0x038C }, { 0x038E, 0x03A1 }, { 0x03A3, 0x03F5 }, { 0x03F7, 0x0481 }, { 0x048A, 0x052F },
  { 0x0531, 0x0556 }, { 0x0559, 0x0559 }, { 0x0560, 0x0588 }, { 0x05D0, 0x05EA }, { 0x05EF, 0x05F2 },
  { 0x0620, 0x064A }, { 0x066E, 0x066F }, { 0x0671, 0x06D3 }, { 0x06D5, 0x06D5 }, { 0x06E5, 0x06E6 },
  { 0x06EE, 0x06EF }, { 0x06FA, 0x06FC }, { 0x06FF, 0x06FF }, { 0x0710, 0x0710 }, { 0x0712, 0x072F },
  { 0x074D, 0x07A5 }, { 0x07B1, 0x07B1 }, { 0x07CA, 0x07EA }, { 0x07F4, 0x07F5 }, { 0x07FA, 0x07FA },
  { 0x0800, 0x0815 }, { 0x081A, 0x081A }, { 0x0824, 0x0824 }, { 0x0828, 0x0828 }, { 0x0840, 0x0858 },
  { 0x0860, 0x086A }, { 0x0870, 0x0887 }, { 0x0889, 0x088E }, { 0x08A0, 0x08C9 }, { 0x0904, 0x0939 },
  { 0x093D, 0x093D }, { 0x0950, 0x0950 }, { 0x0958, 0x0961 }, { 0x0971, 0x0980 }, { 0x0985, 0x098C },
  { 0x098F, 0x0990 }, { 0x0993, 0x09A8 }, { 0x09AA, 0x09B0 }, { 0x09B2, 0x09B2 }, { 0x09B6, 0x09B9 },
  { 0x09BD, 0x09BD }, { 0x09CE, 0x09CE }, { 0x09DC, 0x09DD }, { 0x09DF, 0x09E1 }, { 0x09F0, 0x09F1 },
  { 0x09FC, 0x09FC }, { 0x0A05, 0x0A0A }, { 0x0A0F, 0x0A10 }, { 0x0A13, 0x0A28 }, { 0x0A2A, 0x0A30 },
  { 0x0A32, 0x0A33 }, { 0x0A35, 0x0A36 }, { 0x0A38, 0x0A39 }, { 0x0A59, 0x0A5C }, { 0x0A5E, 0x0A5E },
  { 0x0A72, 0x0A74 }, { 0x0A85, 0x0A8D }, { 0x0A8F, 0x0A91 }, { 0x0A93, 0x0AA8 }, { 0x0AAA, 0x0AB0 },
  { 0x0AB2, 0x0AB3 }, { 0x0AB5, 0x0AB9 }, { 0x0ABD, 0x0ABD }, { 0x0AD0, 0x0AD0 }, { 0x0AE0, 0x0AE1 },
  { 0x0AF9, 0x0AF9 }, { 0x0B05, 0x0B0C }, { 0x0B0F, 0x0B10 }, { 0x0B13, 0x0B28 }, { 0x0B2A, 0x0B30 },
  { 0x0B32, 0x0B33 }, { 0x0B35, 0x0B39 }, { 0x0B3D, 0x0B3D }, { 0x0B5C, 0x0B5D }, { 0x0B5F, 0x0B61 },
  { 0x0B71, 0x0B71 }, { 0x0B83, 0x0B83 }, { 0x0B85, 0x0B8A }, { 0x0B8E, 0x0B90 }, { 0x0B92, 0x0B95 },
  { 0x0B99, 0x0B9A }, { 0x0B9C, 0x0B9C }, { 0x0B9E, 0x0B9F }, { 0x0BA3, 0x0BA4 }, { 0x0BA8, 0x0BAA },
  { 0x0BAE, 0x0BB9 }, { 0x0BD0, 0x0BD0 }, { 0x0C05, 0x0C0C }, { 0x0C0E, 0x0C10 }, { 0x0C12, 0x0C28 },
  { 0x0C2A, 0x0C39 }, { 0x0C3D, 0x0C3D }, { 0x0C58, 0x0C5A }, { 0x0C5D, 0x0C5D }, { 0x0C60, 0x0C61 },
  { 0x0C80, 0x0C80 }, { 0x0C85, 0x0C8C }, { 0x0C8E, 0x0C90 }, { 0x0C92, 0x0CA8 }, { 0x0CAA, 0x0CB3 },
  { 0x0CB5, 0x0CB9 }, { 0x0CBD, 0x0CBD }, { 0x0CDD, 0x0CDE }, { 0x0CE0, 0x0CE1 }, { 0x0CF1, 0x0CF2 },
  { 0x0D04, 0x0D0C }, { 0x0D0E, 0x0D10 }, { 0x0D12, 0x0D3A }, { 0x0D3D, 0x0D3D }, { 0x0D4E, 0x0D4E },
  { 0x0D54, 0x0D56 }, { 0x0D5F, 0x0D61 }, { 0x0D7A, 0x0D7F }, { 0x0D85, 0x0D96 }, { 0x0D9A, 0x0DB1 },
  { 0x0DB3, 0x0DBB }, { 0x0DBD, 0x0DBD }, { 0x0DC0, 0x0DC6 }, { 0x0E01, 0x0E30 }, { 0x0E32, 0x0E33 },
  { 0x0E40, 0x0E46 }, { 0x0E81, 0x0E82 }, { 0x0E84, 0x0E84 }, { 0x0E86, 0x0E8A }, { 0x0E8C, 0x0EA3 },
  { 0x0EA5, 0x0EA5 }, { 0x0EA7, 0x0EB0 }, { 0x0EB2, 0x0EB3 }, { 0x0EBD, 0x0EBD }, { 0x0EC0, 0x0EC4 },
  { 0x0EC6, 0x0EC6 }, { 0x0EDC, 0x0EDF }, { 0x0F00, 0x0F00 }, { 0x0F40, 0x0F47 }, { 0x0F49, 0x0F6C },
  { 0x0F88, 0x0F8C }, { 0x1000, 0x102A }, { 0x103F, 0x103F }, { 0x1050, 0x1055 }, { 0x105A, 0x105D },
  { 0x1061, 0x1061 }, { 0x1065, 0x1066 }, { 0x106E, 0x1070 }, { 0x1075, 0x1081 }, { 0x108E, 0x108E },
  { 0x10A0, 0x10C5 }, { 0x10C7, 0x10C7 }, { 0x10CD, 0x10CD }, { 0x10D0, 0x10FA }, { 0x10FC, 0x1248 },
  { 0x124A, 0x124D }, { 0x1250, 0x1256 }, { 0x1258, 0x1258 }, { 0x125A, 0x125D }, { 0x1260, 0x1288 },
  { 0x128A, 0x128D }, { 0x1290, 0x12B0 }, { 0x12B2, 0x12B5 }, { 0x12B8, 0x12BE }, { 0x12C0, 0x12C0 },
  { 0x12C2, 0x12C5 }, { 0x12C8, 0x12D6 }, { 0x12D8, 0x1310 }, { 0x1312, 0x1315 }, { 0x1318, 0x135A },
  { 0x1380, 0x138F }, { 0x13A0, 0x13F5 }, { 0x13F8, 0x13FD }, { 0x1401, 0x166C }, { 0x166F, 0x167F },
  { 0x1681, 0x169A }, { 0x16A0, 0x16EA }, { 0x16EE, 0x16F8 }, { 0x1700, 0x1711 }, { 0x171F, 0x1731 },
  { 0x1740, 0x1751 }, { 0x1760, 0x176C }, { 0x176E, 0x1770 }, { 0x1780, 0x17B3 }, { 0x17D7, 0x17D7 },
  { 0x17DC, 0x17DC }, { 0x1820, 0x1878 }, { 0x1880, 0x1884 }, { 0x1887, 0x18A8 }, { 0x18AA, 0x18AA },
  { 0x18B0, 0x18F5 }, { 0x1900, 0x191E }, { 0x1950, 0x196D }, { 0x1970, 0x1974 }, { 0x1980, 0x19AB },
  { 0x19B0, 0x19C9 }, { 0x1A00, 0x1A16 }, { 0x1A20, 0x1A54 }, { 0x1AA7, 0x1AA7 }, { 0x1B05, 0x1B33 },
  { 0x1B45, 0x1B4C }, { 0x1B83, 0x1BA0 }, { 0x1BAE, 0x1BAF }, { 0x1BBA, 0x1BE5 }, { 0x1C00, 0x1C23 },
  { 0x1C4D, 0x1C4F }, { 0x1C5A, 0x1C7D }, { 0x1C80, 0x1C8A }, { 0x1C90, 0x1CBA }, { 0x1CBD, 0x1CBF },
  { 0x1CE9, 0x1CEC }, { 0x1CEE, 0x1CF3 }, { 0x1CF5, 0x1CF6 }, { 0x1CFA, 0x1CFA }, { 0x1D00, 0x1DBF },
  { 0x1E00, 0x1F15 }, { 0x1F18, 0x1F1D }, { 0x1F20, 0x1F45 }, { 0x1F48, 0x1F4D }, { 0x1F50, 0x1F57 },
  { 0x1F59, 0x1F59 }, { 0x1F5B, 0x1F5B }, { 0x1F5D, 0x1F5D }, { 0x1F5F, 0x1F7D }, { 0x1F80, 0x1FB4 },
  { 0x1FB6, 0x1FBC }, { 0x1FBE, 0x1FBE }, { 0x1FC2, 0x1FC4 }, { 0x1FC6, 0x1FCC }, { 0x1FD0, 0x1FD3 },
  { 0x1FD6, 0x1FDB }, { 0x1FE0, 0x1FEC }, { 0x1FF2, 0x1FF4 }, { 0x1FF6, 0x1FFC }, { 0x2071, 0x2071 },
  { 0x207F, 0x207F }, { 0x2090, 0x209C }, { 0x2102, 0x2102 }, { 0x2107, 0x2107 }, { 0x210A, 0x2113 },
  { 0x2115, 0x2115 }, { 0x2119, 0x211D }, { 0x2124, 0x2124 }, { 0x2126, 0x2126 }, { 0x2128, 0x2128 },
  { 0x212A, 0x212D }, { 0x212F, 0x2139 }, { 0x213C, 0x213F }, { 0x2145, 0x2149 }, { 0x214E, 0x214E },
  { 0x2160, 0x2188 }, { 0x2C00, 0x2CE4 }, { 0x2CEB, 0x2CEE }, { 0x2CF2, 0x2CF3 }, { 0x2D00, 0x2D25 },
  { 0x2D27, 0x2D27 }, { 0x2D2D, 0x2D2D }, { 0x2D30, 0x2D67 }, { 0x2D6F, 0x2D6F }, { 0x2D80, 0x2D96 },
  { 0x2DA0, 0x2DA6 }, { 0x2DA8, 0x2DAE }, { 0x2DB0, 0x2DB6 }, { 0x2DB8, 0x2DBE }, { 0x2DC0, 0x2DC6 },
  { 0x2DC8, 0x2DCE }, { 0x2DD0, 0x2DD6 }, { 0x2DD8, 0x2DDE }, { 0x2E2F, 0x2E2F }, { 0x3005, 0x3007 },
  { 0x3021, 0x3029 }, { 0x3031, 0x3035 }, { 0x3038, 0x303C }, { 0x3041, 0x3096 }, { 0x309D, 0x309F },
  { 0x30A1, 0x30FA }, { 0x30FC, 0x30FF }, { 0x3105, 0x312F }, { 0x3131, 0x318E }, { 0x31A0, 0x31BF },
  { 0x31F0, 0x31FF }, { 0x3400, 0x4DBF }, { 0x4E00, 0xA48C }, { 0xA4D0, 0xA4FD }, { 0xA500, 0xA60C },
  { 0xA610, 0xA61F }, { 0xA62A, 0xA62B }, { 0xA640, 0xA66E }, { 0xA67F, 0xA69D }, { 0xA6A0, 0xA6EF },
  { 0xA717, 0xA71F }, { 0xA722, 0xA788 }, { 0xA78B, 0xA7CD }, { 0xA7D0, 0xA7D1 }, { 0xA7D3, 0xA7D3 },
  { 0xA7D5, 0xA7DC }, { 0xA7F2, 0xA801 }, { 0xA803, 0xA805 }, { 0xA807, 0xA80A }, { 0xA80C, 0xA822 },
  { 0xA840, 0xA873 }, { 0xA882, 0xA8B3 }, { 0xA8F2, 0xA8F7 }, { 0xA8FB, 0xA8FB }, { 0xA8FD, 0xA8FE },
  { 0xA90A, 0xA925 }, { 0xA930, 0xA946 }, { 0xA960, 0xA97C }, { 0xA984, 0xA9B2 }, { 0xA9CF, 0xA9CF },
  { 0xA9E0, 0xA9E4 }, { 0xA9E6, 0xA9EF }, { 0xA9FA, 0xA9FE }, { 0xAA00, 0xAA28 }, { 0xAA40, 0xAA42 },
  { 0xAA44, 0xAA4B }, { 0xAA60, 0xAA76 }, { 0xAA7A, 0xAA7A }, { 0xAA7E, 0xAAAF }, { 0xAAB1, 0xAAB1 },
  { 0xAAB5, 0xAAB6 }, { 0xAAB9, 0xAABD }, { 0xAAC0, 0xAAC0 }, { 0xAAC2, 0xAAC2 }, { 0xAADB, 0xAADD },
  { 0xAAE0, 0xAAEA }, { 0xAAF2, 0xAAF4 }, { 0xAB01, 0xAB06 }, { 0xAB09, 0xAB0E }, { 0xAB11, 0xAB16 },
  { 0xAB20, 0xAB26 }, { 0xAB28, 0xAB2E }, { 0xAB30, 0xAB5A }, { 0xAB5C, 0xAB69 }, { 0xAB70, 0xABE2 },
  { 0xAC00, 0xD7A3 }, { 0xD7B0, 0xD7C6 }, { 0xD7CB, 0xD7FB }, { 0xF900, 0xFA6D }, { 0xFA70, 0xFAD9 },
  { 0xFB00, 0xFB06 }, { 0xFB13, 0xFB17 }, { 0xFB1D, 0xFB1D }, { 0xFB1F, 0xFB28 }, { 0xFB2A, 0xFB36 },
  { 0xFB38, 0xFB3C }, { 0xFB3E, 0xFB3E }, { 0xFB40, 0xFB41 }, { 0xFB43, 0xFB44 }, { 0xFB46, 0xFBB1 },
  { 0xFBD3, 0xFD3D }, { 0xFD50, 0xFD8F }, { 0xFD92, 0xFDC7 }, { 0xFDF0, 0xFDFB }, { 0xFE70, 0xFE74 },
  { 0xFE76, 0xFEFC }, { 0xFF21, 0xFF3A }, { 0xFF41, 0xFF5A }, { 0xFF66, 0xFFBE }, { 0xFFC2, 0xFFC7 },
  { 0xFFCA, 0xFFCF }, { 0xFFD2, 0xFFD7 }, { 0xFFDA, 0xFFDC }
};
EFJSON_PRIVATE const efjsonUint16 EFJSON__IDENTIFIER_START2[][2] = {
  { 0x0000, 0x000B }, { 0x000D, 0x0026 }, { 0x0028, 0x003A }, { 0x003C, 0x003D }, { 0x003F, 0x004D },
  { 0x0050, 0x005D }, { 0x0080, 0x00FA }, { 0x0140, 0x0174 }, { 0x0280, 0x029C }, { 0x02A0, 0x02D0 },
  { 0x0300, 0x031F }, { 0x032D, 0x034A }, { 0x0350, 0x0375 }, { 0x0380, 0x039D }, { 0x03A0, 0x03C3 },
  { 0x03C8, 0x03CF }, { 0x03D1, 0x03D5 }, { 0x0400, 0x049D }, { 0x04B0, 0x04D3 }, { 0x04D8, 0x04FB },
  { 0x0500, 0x0527 }, { 0x0530, 0x0563 }, { 0x0570, 0x057A }, { 0x057C, 0x058A }, { 0x058C, 0x0592 },
  { 0x0594, 0x0595 }, { 0x0597, 0x05A1 }, { 0x05A3, 0x05B1 }, { 0x05B3, 0x05B9 }, { 0x05BB, 0x05BC },
  { 0x05C0, 0x05F3 }, { 0x0600, 0x0736 }, { 0x0740, 0x0755 }, { 0x0760, 0x0767 }, { 0x0780, 0x0785 },
  { 0x0787, 0x07B0 }, { 0x07B2, 0x07BA }, { 0x0800, 0x0805 }, { 0x0808, 0x0808 }, { 0x080A, 0x0835 },
  { 0x0837, 0x0838 }, { 0x083C, 0x083C }, { 0x083F, 0x0855 }, { 0x0860, 0x0876 }, { 0x0880, 0x089E },
  { 0x08E0, 0x08F2 }, { 0x08F4, 0x08F5 }, { 0x0900, 0x0915 }, { 0x0920, 0x0939 }, { 0x0980, 0x09B7 },
  { 0x09BE, 0x09BF }, { 0x0A00, 0x0A00 }, { 0x0A10, 0x0A13 }, { 0x0A15, 0x0A17 }, { 0x0A19, 0x0A35 },
  { 0x0A60, 0x0A7C }, { 0x0A80, 0x0A9C }, { 0x0AC0, 0x0AC7 }, { 0x0AC9, 0x0AE4 }, { 0x0B00, 0x0B35 },
  { 0x0B40, 0x0B55 }, { 0x0B60, 0x0B72 }, { 0x0B80, 0x0B91 }, { 0x0C00, 0x0C48 }, { 0x0C80, 0x0CB2 },
  { 0x0CC0, 0x0CF2 }, { 0x0D00, 0x0D23 }, { 0x0D4A, 0x0D65 }, { 0x0D6F, 0x0D85 }, { 0x0E80, 0x0EA9 },
  { 0x0EB0, 0x0EB1 }, { 0x0EC2, 0x0EC4 }, { 0x0F00, 0x0F1C }, { 0x0F27, 0x0F27 }, { 0x0F30, 0x0F45 },
  { 0x0F70, 0x0F81 }, { 0x0FB0, 0x0FC4 }, { 0x0FE0, 0x0FF6 }, { 0x1003, 0x1037 }, { 0x1071, 0x1072 },
  { 0x1075, 0x1075 }, { 0x1083, 0x10AF }, { 0x10D0, 0x10E8 }, { 0x1103, 0x1126 }, { 0x1144, 0x1144 },
  { 0x1147, 0x1147 }, { 0x1150, 0x1172 }, { 0x1176, 0x1176 }, { 0x1183, 0x11B2 }, { 0x11C1, 0x11C4 },
  { 0x11DA, 0x11DA }, { 0x11DC, 0x11DC }, { 0x1200, 0x1211 }, { 0x1213, 0x122B }, { 0x123F, 0x1240 },
  { 0x1280, 0x1286 }, { 0x1288, 0x1288 }, { 0x128A, 0x128D }, { 0x128F, 0x129D }, { 0x129F, 0x12A8 },
  { 0x12B0, 0x12DE }, { 0x1305, 0x130C }, { 0x130F, 0x1310 }, { 0x1313, 0x1328 }, { 0x132A, 0x1330 },
  { 0x1332, 0x1333 }, { 0x1335, 0x1339 }, { 0x133D, 0x133D }, { 0x1350, 0x1350 }, { 0x135D, 0x1361 },
  { 0x1380, 0x1389 }, { 0x138B, 0x138B }, { 0x138E, 0x138E }, { 0x1390, 0x13B5 }, { 0x13B7, 0x13B7 },
  { 0x13D1, 0x13D1 }, { 0x13D3, 0x13D3 }, { 0x1400, 0x1434 }, { 0x1447, 0x144A }, { 0x145F, 0x1461 },
  { 0x1480, 0x14AF }, { 0x14C4, 0x14C5 }, { 0x14C7, 0x14C7 }, { 0x1580, 0x15AE }, { 0x15D8, 0x15DB },
  { 0x1600, 0x162F }, { 0x1644, 0x1644 }, { 0x1680, 0x16AA }, { 0x16B8, 0x16B8 }, { 0x1700, 0x171A },
  { 0x1740, 0x1746 }, { 0x1800, 0x182B }, { 0x18A0, 0x18DF }, { 0x18FF, 0x1906 }, { 0x1909, 0x1909 },
  { 0x190C, 0x1913 }, { 0x1915, 0x1916 }, { 0x1918, 0x192F }, { 0x193F, 0x193F }, { 0x1941, 0x1941 },
  { 0x19A0, 0x19A7 }, { 0x19AA, 0x19D0 }, { 0x19E1, 0x19E1 }, { 0x19E3, 0x19E3 }, { 0x1A00, 0x1A00 },
  { 0x1A0B, 0x1A32 }, { 0x1A3A, 0x1A3A }, { 0x1A50, 0x1A50 }, { 0x1A5C, 0x1A89 }, { 0x1A9D, 0x1A9D },
  { 0x1AB0, 0x1AF8 }, { 0x1BC0, 0x1BE0 }, { 0x1C00, 0x1C08 }, { 0x1C0A, 0x1C2E }, { 0x1C40, 0x1C40 },
  { 0x1C72, 0x1C8F }, { 0x1D00, 0x1D06 }, { 0x1D08, 0x1D09 }, { 0x1D0B, 0x1D30 }, { 0x1D46, 0x1D46 },
  { 0x1D60, 0x1D65 }, { 0x1D67, 0x1D68 }, { 0x1D6A, 0x1D89 }, { 0x1D98, 0x1D98 }, { 0x1EE0, 0x1EF2 },
  { 0x1F02, 0x1F02 }, { 0x1F04, 0x1F10 }, { 0x1F12, 0x1F33 }, { 0x1FB0, 0x1FB0 }, { 0x2000, 0x2399 },
  { 0x2400, 0x246E }, { 0x2480, 0x2543 }, { 0x2F90, 0x2FF0 }, { 0x3000, 0x342F }, { 0x3441, 0x3446 },
  { 0x3460, 0x43FA }, { 0x4400, 0x4646 }, { 0x6100, 0x611D }, { 0x6800, 0x6A38 }, { 0x6A40, 0x6A5E },
  { 0x6A70, 0x6ABE }, { 0x6AD0, 0x6AED }, { 0x6B00, 0x6B2F }, { 0x6B40, 0x6B43 }, { 0x6B63, 0x6B77 },
  { 0x6B7D, 0x6B8F }, { 0x6D40, 0x6D6C }, { 0x6E40, 0x6E7F }, { 0x6F00, 0x6F4A }, { 0x6F50, 0x6F50 },
  { 0x6F93, 0x6F9F }, { 0x6FE0, 0x6FE1 }, { 0x6FE3, 0x6FE3 }, { 0x7000, 0x87F7 }, { 0x8800, 0x8CD5 },
  { 0x8CFF, 0x8D08 }, { 0xAFF0, 0xAFF3 }, { 0xAFF5, 0xAFFB }, { 0xAFFD, 0xAFFE }, { 0xB000, 0xB122 },
  { 0xB132, 0xB132 }, { 0xB150, 0xB152 }, { 0xB155, 0xB155 }, { 0xB164, 0xB167 }, { 0xB170, 0xB2FB },
  { 0xBC00, 0xBC6A }, { 0xBC70, 0xBC7C }, { 0xBC80, 0xBC88 }, { 0xBC90, 0xBC99 }, { 0xD400, 0xD454 },
  { 0xD456, 0xD49C }, { 0xD49E, 0xD49F }, { 0xD4A2, 0xD4A2 }, { 0xD4A5, 0xD4A6 }, { 0xD4A9, 0xD4AC },
  { 0xD4AE, 0xD4B9 }, { 0xD4BB, 0xD4BB }, { 0xD4BD, 0xD4C3 }, { 0xD4C5, 0xD505 }, { 0xD507, 0xD50A },
  { 0xD50D, 0xD514 }, { 0xD516, 0xD51C }, { 0xD51E, 0xD539 }, { 0xD53B, 0xD53E }, { 0xD540, 0xD544 },
  { 0xD546, 0xD546 }, { 0xD54A, 0xD550 }, { 0xD552, 0xD6A5 }, { 0xD6A8, 0xD6C0 }, { 0xD6C2, 0xD6DA },
  { 0xD6DC, 0xD6FA }, { 0xD6FC, 0xD714 }, { 0xD716, 0xD734 }, { 0xD736, 0xD74E }, { 0xD750, 0xD76E },
  { 0xD770, 0xD788 }, { 0xD78A, 0xD7A8 }, { 0xD7AA, 0xD7C2 }, { 0xD7C4, 0xD7CB }, { 0xDF00, 0xDF1E },
  { 0xDF25, 0xDF2A }, { 0xE030, 0xE06D }, { 0xE100, 0xE12C }, { 0xE137, 0xE13D }, { 0xE14E, 0xE14E },
  { 0xE290, 0xE2AD }, { 0xE2C0, 0xE2EB }, { 0xE4D0, 0xE4EB }, { 0xE5D0, 0xE5ED }, { 0xE5F0, 0xE5F0 },
  { 0xE7E0, 0xE7E6 }, { 0xE7E8, 0xE7EB }, { 0xE7ED, 0xE7EE }, { 0xE7F0, 0xE7FE }, { 0xE800, 0xE8C4 },
  { 0xE900, 0xE943 }, { 0xE94B, 0xE94B }, { 0xEE00, 0xEE03 }, { 0xEE05, 0xEE1F }, { 0xEE21, 0xEE22 },
  { 0xEE24, 0xEE24 }, { 0xEE27, 0xEE27 }, { 0xEE29, 0xEE32 }, { 0xEE34, 0xEE37 }, { 0xEE39, 0xEE39 },
  { 0xEE3B, 0xEE3B }, { 0xEE42, 0xEE42 }, { 0xEE47, 0xEE47 }, { 0xEE49, 0xEE49 }, { 0xEE4B, 0xEE4B },
  { 0xEE4D, 0xEE4F }, { 0xEE51, 0xEE52 }, { 0xEE54, 0xEE54 }, { 0xEE57, 0xEE57 }, { 0xEE59, 0xEE59 },
  { 0xEE5B, 0xEE5B }, { 0xEE5D, 0xEE5D }, { 0xEE5F, 0xEE5F }, { 0xEE61, 0xEE62 }, { 0xEE64, 0xEE64 },
  { 0xEE67, 0xEE6A }, { 0xEE6C, 0xEE72 }, { 0xEE74, 0xEE77 }, { 0xEE79, 0xEE7C }, { 0xEE7E, 0xEE7E },
  { 0xEE80, 0xEE89 }, { 0xEE8B, 0xEE9B }, { 0xEEA1, 0xEEA3 }, { 0xEEA5, 0xEEA9 }, { 0xEEAB, 0xEEBB },

};
EFJSON_PRIVATE const efjsonUint32 EFJSON__IDENTIFIER_START3[][2] = {
  { 0x20000, 0x2A6DF }, { 0x2A700, 0x2B739 }, { 0x2B740, 0x2B81D }, { 0x2B820, 0x2CEA1 }, { 0x2CEB0, 0x2EBE0 },
  { 0x2EBF0, 0x2EE5D }, { 0x2F800, 0x2FA1D }, { 0x30000, 0x3134A }, { 0x31350, 0x323AF }
};

EFJSON_PRIVATE const efjsonUint16 EFJSON__IDENTIFIER_NEXT_DELTA1[][2] = {
  { 0x0030, 0x0039 }, { 0x0300, 0x036F }, { 0x0483, 0x0487 }, { 0x0591, 0x05BD }, { 0x05BF, 0x05BF },
  { 0x05C1, 0x05C2 }, { 0x05C4, 0x05C5 }, { 0x05C7, 0x05C7 }, { 0x0610, 0x061A }, { 0x064B, 0x0669 },
  { 0x0670, 0x0670 }, { 0x06D6, 0x06DC }, { 0x06DF, 0x06E4 }, { 0x06E7, 0x06E8 }, { 0x06EA, 0x06ED },
  { 0x06F0, 0x06F9 }, { 0x0711, 0x0711 }, { 0x0730, 0x074A }, { 0x07A6, 0x07B0 }, { 0x07C0, 0x07C9 },
  { 0x07EB, 0x07F3 }, { 0x07FD, 0x07FD }, { 0x0816, 0x0819 }, { 0x081B, 0x0823 }, { 0x0825, 0x0827 },
  { 0x0829, 0x082D }, { 0x0859, 0x085B }, { 0x0897, 0x089F }, { 0x08CA, 0x08E1 }, { 0x08E3, 0x0903 },
  { 0x093A, 0x093C }, { 0x093E, 0x094F }, { 0x0951, 0x0957 }, { 0x0962, 0x0963 }, { 0x0966, 0x096F },
  { 0x0981, 0x0983 }, { 0x09BC, 0x09BC }, { 0x09BE, 0x09C4 }, { 0x09C7, 0x09C8 }, { 0x09CB, 0x09CD },
  { 0x09D7, 0x09D7 }, { 0x09E2, 0x09E3 }, { 0x09E6, 0x09EF }, { 0x09FE, 0x09FE }, { 0x0A01, 0x0A03 },
  { 0x0A3C, 0x0A3C }, { 0x0A3E, 0x0A42 }, { 0x0A47, 0x0A48 }, { 0x0A4B, 0x0A4D }, { 0x0A51, 0x0A51 },
  { 0x0A66, 0x0A71 }, { 0x0A75, 0x0A75 }, { 0x0A81, 0x0A83 }, { 0x0ABC, 0x0ABC }, { 0x0ABE, 0x0AC5 },
  { 0x0AC7, 0x0AC9 }, { 0x0ACB, 0x0ACD }, { 0x0AE2, 0x0AE3 }, { 0x0AE6, 0x0AEF }, { 0x0AFA, 0x0AFF },
  { 0x0B01, 0x0B03 }, { 0x0B3C, 0x0B3C }, { 0x0B3E, 0x0B44 }, { 0x0B47, 0x0B48 }, { 0x0B4B, 0x0B4D },
  { 0x0B55, 0x0B57 }, { 0x0B62, 0x0B63 }, { 0x0B66, 0x0B6F }, { 0x0B82, 0x0B82 }, { 0x0BBE, 0x0BC2 },
  { 0x0BC6, 0x0BC8 }, { 0x0BCA, 0x0BCD }, { 0x0BD7, 0x0BD7 }, { 0x0BE6, 0x0BEF }, { 0x0C00, 0x0C04 },
  { 0x0C3C, 0x0C3C }, { 0x0C3E, 0x0C44 }, { 0x0C46, 0x0C48 }, { 0x0C4A, 0x0C4D }, { 0x0C55, 0x0C56 },
  { 0x0C62, 0x0C63 }, { 0x0C66, 0x0C6F }, { 0x0C81, 0x0C83 }, { 0x0CBC, 0x0CBC }, { 0x0CBE, 0x0CC4 },
  { 0x0CC6, 0x0CC8 }, { 0x0CCA, 0x0CCD }, { 0x0CD5, 0x0CD6 }, { 0x0CE2, 0x0CE3 }, { 0x0CE6, 0x0CEF },
  { 0x0CF3, 0x0CF3 }, { 0x0D00, 0x0D03 }, { 0x0D3B, 0x0D3C }, { 0x0D3E, 0x0D44 }, { 0x0D46, 0x0D48 },
  { 0x0D4A, 0x0D4D }, { 0x0D57, 0x0D57 }, { 0x0D62, 0x0D63 }, { 0x0D66, 0x0D6F }, { 0x0D81, 0x0D83 },
  { 0x0DCA, 0x0DCA }, { 0x0DCF, 0x0DD4 }, { 0x0DD6, 0x0DD6 }, { 0x0DD8, 0x0DDF }, { 0x0DE6, 0x0DEF },
  { 0x0DF2, 0x0DF3 }, { 0x0E31, 0x0E31 }, { 0x0E34, 0x0E3A }, { 0x0E47, 0x0E4E }, { 0x0E50, 0x0E59 },
  { 0x0EB1, 0x0EB1 }, { 0x0EB4, 0x0EBC }, { 0x0EC8, 0x0ECE }, { 0x0ED0, 0x0ED9 }, { 0x0F18, 0x0F19 },
  { 0x0F20, 0x0F29 }, { 0x0F35, 0x0F35 }, { 0x0F37, 0x0F37 }, { 0x0F39, 0x0F39 }, { 0x0F3E, 0x0F3F },
  { 0x0F71, 0x0F84 }, { 0x0F86, 0x0F87 }, { 0x0F8D, 0x0F97 }, { 0x0F99, 0x0FBC }, { 0x0FC6, 0x0FC6 },
  { 0x102B, 0x103E }, { 0x1040, 0x1049 }, { 0x1056, 0x1059 }, { 0x105E, 0x1060 }, { 0x1062, 0x1064 },
  { 0x1067, 0x106D }, { 0x1071, 0x1074 }, { 0x1082, 0x108D }, { 0x108F, 0x109D }, { 0x135D, 0x135F },
  { 0x1712, 0x1715 }, { 0x1732, 0x1734 }, { 0x1752, 0x1753 }, { 0x1772, 0x1773 }, { 0x17B4, 0x17D3 },
  { 0x17DD, 0x17DD }, { 0x17E0, 0x17E9 }, { 0x180B, 0x180D }, { 0x180F, 0x1819 }, { 0x1885, 0x1886 },
  { 0x18A9, 0x18A9 }, { 0x1920, 0x192B }, { 0x1930, 0x193B }, { 0x1946, 0x194F }, { 0x19D0, 0x19D9 },
  { 0x1A17, 0x1A1B }, { 0x1A55, 0x1A5E }, { 0x1A60, 0x1A7C }, { 0x1A7F, 0x1A89 }, { 0x1A90, 0x1A99 },
  { 0x1AB0, 0x1ABD }, { 0x1ABF, 0x1ACE }, { 0x1B00, 0x1B04 }, { 0x1B34, 0x1B44 }, { 0x1B50, 0x1B59 },
  { 0x1B6B, 0x1B73 }, { 0x1B80, 0x1B82 }, { 0x1BA1, 0x1BAD }, { 0x1BB0, 0x1BB9 }, { 0x1BE6, 0x1BF3 },
  { 0x1C24, 0x1C37 }, { 0x1C40, 0x1C49 }, { 0x1C50, 0x1C59 }, { 0x1CD0, 0x1CD2 }, { 0x1CD4, 0x1CE8 },
  { 0x1CED, 0x1CED }, { 0x1CF4, 0x1CF4 }, { 0x1CF7, 0x1CF9 }, { 0x1DC0, 0x1DFF }, { 0x200C, 0x200D },
  { 0x203F, 0x2040 }, { 0x2054, 0x2054 }, { 0x20D0, 0x20DC }, { 0x20E1, 0x20E1 }, { 0x20E5, 0x20F0 },
  { 0x2CEF, 0x2CF1 }, { 0x2D7F, 0x2D7F }, { 0x2DE0, 0x2DFF }, { 0x302A, 0x302F }, { 0x3099, 0x309A },
  { 0xA620, 0xA629 }, { 0xA66F, 0xA66F }, { 0xA674, 0xA67D }, { 0xA69E, 0xA69F }, { 0xA6F0, 0xA6F1 },
  { 0xA802, 0xA802 }, { 0xA806, 0xA806 }, { 0xA80B, 0xA80B }, { 0xA823, 0xA827 }, { 0xA82C, 0xA82C },
  { 0xA880, 0xA881 }, { 0xA8B4, 0xA8C5 }, { 0xA8D0, 0xA8D9 }, { 0xA8E0, 0xA8F1 }, { 0xA8FF, 0xA909 },
  { 0xA926, 0xA92D }, { 0xA947, 0xA953 }, { 0xA980, 0xA983 }, { 0xA9B3, 0xA9C0 }, { 0xA9D0, 0xA9D9 },
  { 0xA9E5, 0xA9E5 }, { 0xA9F0, 0xA9F9 }, { 0xAA29, 0xAA36 }, { 0xAA43, 0xAA43 }, { 0xAA4C, 0xAA4D },
  { 0xAA50, 0xAA59 }, { 0xAA7B, 0xAA7D }, { 0xAAB0, 0xAAB0 }, { 0xAAB2, 0xAAB4 }, { 0xAAB7, 0xAAB8 },
  { 0xAABE, 0xAABF }, { 0xAAC1, 0xAAC1 }, { 0xAAEB, 0xAAEF }, { 0xAAF5, 0xAAF6 }, { 0xABE3, 0xABEA },
  { 0xABEC, 0xABED }, { 0xABF0, 0xABF9 }, { 0xFB1E, 0xFB1E }, { 0xFE00, 0xFE0F }, { 0xFE20, 0xFE2F },
  { 0xFE33, 0xFE34 }, { 0xFE4D, 0xFE4F }, { 0xFF10, 0xFF19 }, { 0xFF3F, 0xFF3F }
};
EFJSON_PRIVATE const efjsonUint16 EFJSON__IDENTIFIER_NEXT_DELTA2[][2] = {
  { 0x01FD, 0x01FD }, { 0x02E0, 0x02E0 }, { 0x0376, 0x037A }, { 0x04A0, 0x04A9 }, { 0x0A01, 0x0A03 },
  { 0x0A05, 0x0A06 }, { 0x0A0C, 0x0A0F }, { 0x0A38, 0x0A3A }, { 0x0A3F, 0x0A3F }, { 0x0AE5, 0x0AE6 },
  { 0x0D24, 0x0D27 }, { 0x0D30, 0x0D39 }, { 0x0D40, 0x0D49 }, { 0x0D69, 0x0D6D }, { 0x0EAB, 0x0EAC },
  { 0x0EFC, 0x0EFF }, { 0x0F46, 0x0F50 }, { 0x0F82, 0x0F85 }, { 0x1000, 0x1002 }, { 0x1038, 0x1046 },
  { 0x1066, 0x1070 }, { 0x1073, 0x1074 }, { 0x107F, 0x1082 }, { 0x10B0, 0x10BA }, { 0x10C2, 0x10C2 },
  { 0x10F0, 0x10F9 }, { 0x1100, 0x1102 }, { 0x1127, 0x1134 }, { 0x1136, 0x113F }, { 0x1145, 0x1146 },
  { 0x1173, 0x1173 }, { 0x1180, 0x1182 }, { 0x11B3, 0x11C0 }, { 0x11C9, 0x11CC }, { 0x11CE, 0x11D9 },
  { 0x122C, 0x1237 }, { 0x123E, 0x123E }, { 0x1241, 0x1241 }, { 0x12DF, 0x12EA }, { 0x12F0, 0x12F9 },
  { 0x1300, 0x1303 }, { 0x133B, 0x133C }, { 0x133E, 0x1344 }, { 0x1347, 0x1348 }, { 0x134B, 0x134D },
  { 0x1357, 0x1357 }, { 0x1362, 0x1363 }, { 0x1366, 0x136C }, { 0x1370, 0x1374 }, { 0x13B8, 0x13C0 },
  { 0x13C2, 0x13C2 }, { 0x13C5, 0x13C5 }, { 0x13C7, 0x13CA }, { 0x13CC, 0x13D0 }, { 0x13D2, 0x13D2 },
  { 0x13E1, 0x13E2 }, { 0x1435, 0x1446 }, { 0x1450, 0x1459 }, { 0x145E, 0x145E }, { 0x14B0, 0x14C3 },
  { 0x14D0, 0x14D9 }, { 0x15AF, 0x15B5 }, { 0x15B8, 0x15C0 }, { 0x15DC, 0x15DD }, { 0x1630, 0x1640 },
  { 0x1650, 0x1659 }, { 0x16AB, 0x16B7 }, { 0x16C0, 0x16C9 }, { 0x16D0, 0x16E3 }, { 0x171D, 0x172B },
  { 0x1730, 0x1739 }, { 0x182C, 0x183A }, { 0x18E0, 0x18E9 }, { 0x1930, 0x1935 }, { 0x1937, 0x1938 },
  { 0x193B, 0x193E }, { 0x1940, 0x1940 }, { 0x1942, 0x1943 }, { 0x1950, 0x1959 }, { 0x19D1, 0x19D7 },
  { 0x19DA, 0x19E0 }, { 0x19E4, 0x19E4 }, { 0x1A01, 0x1A0A }, { 0x1A33, 0x1A39 }, { 0x1A3B, 0x1A3E },
  { 0x1A47, 0x1A47 }, { 0x1A51, 0x1A5B }, { 0x1A8A, 0x1A99 }, { 0x1BF0, 0x1BF9 }, { 0x1C2F, 0x1C36 },
  { 0x1C38, 0x1C3F }, { 0x1C50, 0x1C59 }, { 0x1C92, 0x1CA7 }, { 0x1CA9, 0x1CB6 }, { 0x1D31, 0x1D36 },
  { 0x1D3A, 0x1D3A }, { 0x1D3C, 0x1D3D }, { 0x1D3F, 0x1D45 }, { 0x1D47, 0x1D47 }, { 0x1D50, 0x1D59 },
  { 0x1D8A, 0x1D8E }, { 0x1D90, 0x1D91 }, { 0x1D93, 0x1D97 }, { 0x1DA0, 0x1DA9 }, { 0x1EF3, 0x1EF6 },
  { 0x1F00, 0x1F01 }, { 0x1F03, 0x1F03 }, { 0x1F34, 0x1F3A }, { 0x1F3E, 0x1F42 }, { 0x1F50, 0x1F5A },
  { 0x3440, 0x3440 }, { 0x3447, 0x3455 }, { 0x611E, 0x6139 }, { 0x6A60, 0x6A69 }, { 0x6AC0, 0x6AC9 },
  { 0x6AF0, 0x6AF4 }, { 0x6B30, 0x6B36 }, { 0x6B50, 0x6B59 }, { 0x6D70, 0x6D79 }, { 0x6F4F, 0x6F4F },
  { 0x6F51, 0x6F87 }, { 0x6F8F, 0x6F92 }, { 0x6FE4, 0x6FE4 }, { 0x6FF0, 0x6FF1 }, { 0xBC9D, 0xBC9E },
  { 0xCCF0, 0xCCF9 }, { 0xCF00, 0xCF2D }, { 0xCF30, 0xCF46 }, { 0xD165, 0xD169 }, { 0xD16D, 0xD172 },
  { 0xD17B, 0xD182 }, { 0xD185, 0xD18B }, { 0xD1AA, 0xD1AD }, { 0xD242, 0xD244 }, { 0xD7CE, 0xD7FF },
  { 0xDA00, 0xDA36 }, { 0xDA3B, 0xDA6C }, { 0xDA75, 0xDA75 }, { 0xDA84, 0xDA84 }, { 0xDA9B, 0xDA9F },
  { 0xDAA1, 0xDAAF }, { 0xE000, 0xE006 }, { 0xE008, 0xE018 }, { 0xE01B, 0xE021 }, { 0xE023, 0xE024 },
  { 0xE026, 0xE02A }, { 0xE08F, 0xE08F }, { 0xE130, 0xE136 }, { 0xE140, 0xE149 }, { 0xE2AE, 0xE2AE },
  { 0xE2EC, 0xE2F9 }, { 0xE4EC, 0xE4F9 }, { 0xE5EE, 0xE5EF }, { 0xE5F1, 0xE5FA }, { 0xE8D0, 0xE8D6 },
  { 0xE944, 0xE94A }, { 0xE950, 0xE959 }, { 0xFBF0, 0xFBF9 },
};
EFJSON_PRIVATE const efjsonUint32 EFJSON__IDENTIFIER_NEXT_DELTA3[][2] = { { 0xE0100, 0xE01EF } };

    #if EFJSON_CONF_EXPOSE_UNICODE
EFJSON_PRIVATE const efjsonUint16 EFJSON__GRAPH1[][2] = {
  { 0x0020, 0x007E }, { 0x00A0, 0x00AC }, { 0x00AE, 0x0377 }, { 0x037A, 0x037F }, { 0x0384, 0x038A },
  { 0x038C, 0x038C }, { 0x038E, 0x03A1 }, { 0x03A3, 0x052F }, { 0x0531, 0x0556 }, { 0x0559, 0x058A },
  { 0x058D, 0x058F }, { 0x0591, 0x05C7 }, { 0x05D0, 0x05EA }, { 0x05EF, 0x05F4 }, { 0x0606, 0x061B },
  { 0x061D, 0x06DC }, { 0x06DE, 0x070D }, { 0x0710, 0x074A }, { 0x074D, 0x07B1 }, { 0x07C0, 0x07FA },
  { 0x07FD, 0x082D }, { 0x0830, 0x083E }, { 0x0840, 0x085B }, { 0x085E, 0x085E }, { 0x0860, 0x086A },
  { 0x0870, 0x088E }, { 0x0897, 0x08E1 }, { 0x08E3, 0x0983 }, { 0x0985, 0x098C }, { 0x098F, 0x0990 },
  { 0x0993, 0x09A8 }, { 0x09AA, 0x09B0 }, { 0x09B2, 0x09B2 }, { 0x09B6, 0x09B9 }, { 0x09BC, 0x09C4 },
  { 0x09C7, 0x09C8 }, { 0x09CB, 0x09CE }, { 0x09D7, 0x09D7 }, { 0x09DC, 0x09DD }, { 0x09DF, 0x09E3 },
  { 0x09E6, 0x09FE }, { 0x0A01, 0x0A03 }, { 0x0A05, 0x0A0A }, { 0x0A0F, 0x0A10 }, { 0x0A13, 0x0A28 },
  { 0x0A2A, 0x0A30 }, { 0x0A32, 0x0A33 }, { 0x0A35, 0x0A36 }, { 0x0A38, 0x0A39 }, { 0x0A3C, 0x0A3C },
  { 0x0A3E, 0x0A42 }, { 0x0A47, 0x0A48 }, { 0x0A4B, 0x0A4D }, { 0x0A51, 0x0A51 }, { 0x0A59, 0x0A5C },
  { 0x0A5E, 0x0A5E }, { 0x0A66, 0x0A76 }, { 0x0A81, 0x0A83 }, { 0x0A85, 0x0A8D }, { 0x0A8F, 0x0A91 },
  { 0x0A93, 0x0AA8 }, { 0x0AAA, 0x0AB0 }, { 0x0AB2, 0x0AB3 }, { 0x0AB5, 0x0AB9 }, { 0x0ABC, 0x0AC5 },
  { 0x0AC7, 0x0AC9 }, { 0x0ACB, 0x0ACD }, { 0x0AD0, 0x0AD0 }, { 0x0AE0, 0x0AE3 }, { 0x0AE6, 0x0AF1 },
  { 0x0AF9, 0x0AFF }, { 0x0B01, 0x0B03 }, { 0x0B05, 0x0B0C }, { 0x0B0F, 0x0B10 }, { 0x0B13, 0x0B28 },
  { 0x0B2A, 0x0B30 }, { 0x0B32, 0x0B33 }, { 0x0B35, 0x0B39 }, { 0x0B3C, 0x0B44 }, { 0x0B47, 0x0B48 },
  { 0x0B4B, 0x0B4D }, { 0x0B55, 0x0B57 }, { 0x0B5C, 0x0B5D }, { 0x0B5F, 0x0B63 }, { 0x0B66, 0x0B77 },
  { 0x0B82, 0x0B83 }, { 0x0B85, 0x0B8A }, { 0x0B8E, 0x0B90 }, { 0x0B92, 0x0B95 }, { 0x0B99, 0x0B9A },
  { 0x0B9C, 0x0B9C }, { 0x0B9E, 0x0B9F }, { 0x0BA3, 0x0BA4 }, { 0x0BA8, 0x0BAA }, { 0x0BAE, 0x0BB9 },
  { 0x0BBE, 0x0BC2 }, { 0x0BC6, 0x0BC8 }, { 0x0BCA, 0x0BCD }, { 0x0BD0, 0x0BD0 }, { 0x0BD7, 0x0BD7 },
  { 0x0BE6, 0x0BFA }, { 0x0C00, 0x0C0C }, { 0x0C0E, 0x0C10 }, { 0x0C12, 0x0C28 }, { 0x0C2A, 0x0C39 },
  { 0x0C3C, 0x0C44 }, { 0x0C46, 0x0C48 }, { 0x0C4A, 0x0C4D }, { 0x0C55, 0x0C56 }, { 0x0C58, 0x0C5A },
  { 0x0C5D, 0x0C5D }, { 0x0C60, 0x0C63 }, { 0x0C66, 0x0C6F }, { 0x0C77, 0x0C8C }, { 0x0C8E, 0x0C90 },
  { 0x0C92, 0x0CA8 }, { 0x0CAA, 0x0CB3 }, { 0x0CB5, 0x0CB9 }, { 0x0CBC, 0x0CC4 }, { 0x0CC6, 0x0CC8 },
  { 0x0CCA, 0x0CCD }, { 0x0CD5, 0x0CD6 }, { 0x0CDD, 0x0CDE }, { 0x0CE0, 0x0CE3 }, { 0x0CE6, 0x0CEF },
  { 0x0CF1, 0x0CF3 }, { 0x0D00, 0x0D0C }, { 0x0D0E, 0x0D10 }, { 0x0D12, 0x0D44 }, { 0x0D46, 0x0D48 },
  { 0x0D4A, 0x0D4F }, { 0x0D54, 0x0D63 }, { 0x0D66, 0x0D7F }, { 0x0D81, 0x0D83 }, { 0x0D85, 0x0D96 },
  { 0x0D9A, 0x0DB1 }, { 0x0DB3, 0x0DBB }, { 0x0DBD, 0x0DBD }, { 0x0DC0, 0x0DC6 }, { 0x0DCA, 0x0DCA },
  { 0x0DCF, 0x0DD4 }, { 0x0DD6, 0x0DD6 }, { 0x0DD8, 0x0DDF }, { 0x0DE6, 0x0DEF }, { 0x0DF2, 0x0DF4 },
  { 0x0E01, 0x0E3A }, { 0x0E3F, 0x0E5B }, { 0x0E81, 0x0E82 }, { 0x0E84, 0x0E84 }, { 0x0E86, 0x0E8A },
  { 0x0E8C, 0x0EA3 }, { 0x0EA5, 0x0EA5 }, { 0x0EA7, 0x0EBD }, { 0x0EC0, 0x0EC4 }, { 0x0EC6, 0x0EC6 },
  { 0x0EC8, 0x0ECE }, { 0x0ED0, 0x0ED9 }, { 0x0EDC, 0x0EDF }, { 0x0F00, 0x0F47 }, { 0x0F49, 0x0F6C },
  { 0x0F71, 0x0F97 }, { 0x0F99, 0x0FBC }, { 0x0FBE, 0x0FCC }, { 0x0FCE, 0x0FDA }, { 0x1000, 0x10C5 },
  { 0x10C7, 0x10C7 }, { 0x10CD, 0x10CD }, { 0x10D0, 0x1248 }, { 0x124A, 0x124D }, { 0x1250, 0x1256 },
  { 0x1258, 0x1258 }, { 0x125A, 0x125D }, { 0x1260, 0x1288 }, { 0x128A, 0x128D }, { 0x1290, 0x12B0 },
  { 0x12B2, 0x12B5 }, { 0x12B8, 0x12BE }, { 0x12C0, 0x12C0 }, { 0x12C2, 0x12C5 }, { 0x12C8, 0x12D6 },
  { 0x12D8, 0x1310 }, { 0x1312, 0x1315 }, { 0x1318, 0x135A }, { 0x135D, 0x137C }, { 0x1380, 0x1399 },
  { 0x13A0, 0x13F5 }, { 0x13F8, 0x13FD }, { 0x1400, 0x169C }, { 0x16A0, 0x16F8 }, { 0x1700, 0x1715 },
  { 0x171F, 0x1736 }, { 0x1740, 0x1753 }, { 0x1760, 0x176C }, { 0x176E, 0x1770 }, { 0x1772, 0x1773 },
  { 0x1780, 0x17DD }, { 0x17E0, 0x17E9 }, { 0x17F0, 0x17F9 }, { 0x1800, 0x180D }, { 0x180F, 0x1819 },
  { 0x1820, 0x1878 }, { 0x1880, 0x18AA }, { 0x18B0, 0x18F5 }, { 0x1900, 0x191E }, { 0x1920, 0x192B },
  { 0x1930, 0x193B }, { 0x1940, 0x1940 }, { 0x1944, 0x196D }, { 0x1970, 0x1974 }, { 0x1980, 0x19AB },
  { 0x19B0, 0x19C9 }, { 0x19D0, 0x19DA }, { 0x19DE, 0x1A1B }, { 0x1A1E, 0x1A5E }, { 0x1A60, 0x1A7C },
  { 0x1A7F, 0x1A89 }, { 0x1A90, 0x1A99 }, { 0x1AA0, 0x1AAD }, { 0x1AB0, 0x1ACE }, { 0x1B00, 0x1B4C },
  { 0x1B4E, 0x1BF3 }, { 0x1BFC, 0x1C37 }, { 0x1C3B, 0x1C49 }, { 0x1C4D, 0x1C8A }, { 0x1C90, 0x1CBA },
  { 0x1CBD, 0x1CC7 }, { 0x1CD0, 0x1CFA }, { 0x1D00, 0x1F15 }, { 0x1F18, 0x1F1D }, { 0x1F20, 0x1F45 },
  { 0x1F48, 0x1F4D }, { 0x1F50, 0x1F57 }, { 0x1F59, 0x1F59 }, { 0x1F5B, 0x1F5B }, { 0x1F5D, 0x1F5D },
  { 0x1F5F, 0x1F7D }, { 0x1F80, 0x1FB4 }, { 0x1FB6, 0x1FC4 }, { 0x1FC6, 0x1FD3 }, { 0x1FD6, 0x1FDB },
  { 0x1FDD, 0x1FEF }, { 0x1FF2, 0x1FF4 }, { 0x1FF6, 0x1FFE }, { 0x2000, 0x200A }, { 0x2010, 0x2029 },
  { 0x202F, 0x205F }, { 0x2070, 0x2071 }, { 0x2074, 0x208E }, { 0x2090, 0x209C }, { 0x20A0, 0x20C0 },
  { 0x20D0, 0x20F0 }, { 0x2100, 0x218B }, { 0x2190, 0x2429 }, { 0x2440, 0x244A }, { 0x2460, 0x2B73 },
  { 0x2B76, 0x2B95 }, { 0x2B97, 0x2CF3 }, { 0x2CF9, 0x2D25 }, { 0x2D27, 0x2D27 }, { 0x2D2D, 0x2D2D },
  { 0x2D30, 0x2D67 }, { 0x2D6F, 0x2D70 }, { 0x2D7F, 0x2D96 }, { 0x2DA0, 0x2DA6 }, { 0x2DA8, 0x2DAE },
  { 0x2DB0, 0x2DB6 }, { 0x2DB8, 0x2DBE }, { 0x2DC0, 0x2DC6 }, { 0x2DC8, 0x2DCE }, { 0x2DD0, 0x2DD6 },
  { 0x2DD8, 0x2DDE }, { 0x2DE0, 0x2E5D }, { 0x2E80, 0x2E99 }, { 0x2E9B, 0x2EF3 }, { 0x2F00, 0x2FD5 },
  { 0x2FF0, 0x303F }, { 0x3041, 0x3096 }, { 0x3099, 0x30FF }, { 0x3105, 0x312F }, { 0x3131, 0x318E },
  { 0x3190, 0x31E5 }, { 0x31EF, 0x321E }, { 0x3220, 0xA48C }, { 0xA490, 0xA4C6 }, { 0xA4D0, 0xA62B },
  { 0xA640, 0xA6F7 }, { 0xA700, 0xA7CD }, { 0xA7D0, 0xA7D1 }, { 0xA7D3, 0xA7D3 }, { 0xA7D5, 0xA7DC },
  { 0xA7F2, 0xA82C }, { 0xA830, 0xA839 }, { 0xA840, 0xA877 }, { 0xA880, 0xA8C5 }, { 0xA8CE, 0xA8D9 },
  { 0xA8E0, 0xA953 }, { 0xA95F, 0xA97C }, { 0xA980, 0xA9CD }, { 0xA9CF, 0xA9D9 }, { 0xA9DE, 0xA9FE },
  { 0xAA00, 0xAA36 }, { 0xAA40, 0xAA4D }, { 0xAA50, 0xAA59 }, { 0xAA5C, 0xAAC2 }, { 0xAADB, 0xAAF6 },
  { 0xAB01, 0xAB06 }, { 0xAB09, 0xAB0E }, { 0xAB11, 0xAB16 }, { 0xAB20, 0xAB26 }, { 0xAB28, 0xAB2E },
  { 0xAB30, 0xAB6B }, { 0xAB70, 0xABED }, { 0xABF0, 0xABF9 }, { 0xAC00, 0xD7A3 }, { 0xD7B0, 0xD7C6 },
  { 0xD7CB, 0xD7FB }, { 0xF900, 0xFA6D }, { 0xFA70, 0xFAD9 }, { 0xFB00, 0xFB06 }, { 0xFB13, 0xFB17 },
  { 0xFB1D, 0xFB36 }, { 0xFB38, 0xFB3C }, { 0xFB3E, 0xFB3E }, { 0xFB40, 0xFB41 }, { 0xFB43, 0xFB44 },
  { 0xFB46, 0xFBC2 }, { 0xFBD3, 0xFD8F }, { 0xFD92, 0xFDC7 }, { 0xFDCF, 0xFDCF }, { 0xFDF0, 0xFE19 },
  { 0xFE20, 0xFE52 }, { 0xFE54, 0xFE66 }, { 0xFE68, 0xFE6B }, { 0xFE70, 0xFE74 }, { 0xFE76, 0xFEFC },
  { 0xFF01, 0xFFBE }, { 0xFFC2, 0xFFC7 }, { 0xFFCA, 0xFFCF }, { 0xFFD2, 0xFFD7 }, { 0xFFDA, 0xFFDC },
  { 0xFFE0, 0xFFE6 }, { 0xFFE8, 0xFFEE }, { 0xFFFC, 0xFFFD }
};
EFJSON_PRIVATE const efjsonUint16 EFJSON__GRAPH2[][2] = {
  { 0x0000, 0x000B }, { 0x000D, 0x0026 }, { 0x0028, 0x003A }, { 0x003C, 0x003D }, { 0x003F, 0x004D },
  { 0x0050, 0x005D }, { 0x0080, 0x00FA }, { 0x0100, 0x0102 }, { 0x0107, 0x0133 }, { 0x0137, 0x018E },
  { 0x0190, 0x019C }, { 0x01A0, 0x01A0 }, { 0x01D0, 0x01FD }, { 0x0280, 0x029C }, { 0x02A0, 0x02D0 },
  { 0x02E0, 0x02FB }, { 0x0300, 0x0323 }, { 0x032D, 0x034A }, { 0x0350, 0x037A }, { 0x0380, 0x039D },
  { 0x039F, 0x03C3 }, { 0x03C8, 0x03D5 }, { 0x0400, 0x049D }, { 0x04A0, 0x04A9 }, { 0x04B0, 0x04D3 },
  { 0x04D8, 0x04FB }, { 0x0500, 0x0527 }, { 0x0530, 0x0563 }, { 0x056F, 0x057A }, { 0x057C, 0x058A },
  { 0x058C, 0x0592 }, { 0x0594, 0x0595 }, { 0x0597, 0x05A1 }, { 0x05A3, 0x05B1 }, { 0x05B3, 0x05B9 },
  { 0x05BB, 0x05BC }, { 0x05C0, 0x05F3 }, { 0x0600, 0x0736 }, { 0x0740, 0x0755 }, { 0x0760, 0x0767 },
  { 0x0780, 0x0785 }, { 0x0787, 0x07B0 }, { 0x07B2, 0x07BA }, { 0x0800, 0x0805 }, { 0x0808, 0x0808 },
  { 0x080A, 0x0835 }, { 0x0837, 0x0838 }, { 0x083C, 0x083C }, { 0x083F, 0x0855 }, { 0x0857, 0x089E },
  { 0x08A7, 0x08AF }, { 0x08E0, 0x08F2 }, { 0x08F4, 0x08F5 }, { 0x08FB, 0x091B }, { 0x091F, 0x0939 },
  { 0x093F, 0x093F }, { 0x0980, 0x09B7 }, { 0x09BC, 0x09CF }, { 0x09D2, 0x0A03 }, { 0x0A05, 0x0A06 },
  { 0x0A0C, 0x0A13 }, { 0x0A15, 0x0A17 }, { 0x0A19, 0x0A35 }, { 0x0A38, 0x0A3A }, { 0x0A3F, 0x0A48 },
  { 0x0A50, 0x0A58 }, { 0x0A60, 0x0A9F }, { 0x0AC0, 0x0AE6 }, { 0x0AEB, 0x0AF6 }, { 0x0B00, 0x0B35 },
  { 0x0B39, 0x0B55 }, { 0x0B58, 0x0B72 }, { 0x0B78, 0x0B91 }, { 0x0B99, 0x0B9C }, { 0x0BA9, 0x0BAF },
  { 0x0C00, 0x0C48 }, { 0x0C80, 0x0CB2 }, { 0x0CC0, 0x0CF2 }, { 0x0CFA, 0x0D27 }, { 0x0D30, 0x0D39 },
  { 0x0D40, 0x0D65 }, { 0x0D69, 0x0D85 }, { 0x0D8E, 0x0D8F }, { 0x0E60, 0x0E7E }, { 0x0E80, 0x0EA9 },
  { 0x0EAB, 0x0EAD }, { 0x0EB0, 0x0EB1 }, { 0x0EC2, 0x0EC4 }, { 0x0EFC, 0x0F27 }, { 0x0F30, 0x0F59 },
  { 0x0F70, 0x0F89 }, { 0x0FB0, 0x0FCB }, { 0x0FE0, 0x0FF6 }, { 0x1000, 0x104D }, { 0x1052, 0x1075 },
  { 0x107F, 0x10BC }, { 0x10BE, 0x10C2 }, { 0x10D0, 0x10E8 }, { 0x10F0, 0x10F9 }, { 0x1100, 0x1134 },
  { 0x1136, 0x1147 }, { 0x1150, 0x1176 }, { 0x1180, 0x11DF }, { 0x11E1, 0x11F4 }, { 0x1200, 0x1211 },
  { 0x1213, 0x1241 }, { 0x1280, 0x1286 }, { 0x1288, 0x1288 }, { 0x128A, 0x128D }, { 0x128F, 0x129D },
  { 0x129F, 0x12A9 }, { 0x12B0, 0x12EA }, { 0x12F0, 0x12F9 }, { 0x1300, 0x1303 }, { 0x1305, 0x130C },
  { 0x130F, 0x1310 }, { 0x1313, 0x1328 }, { 0x132A, 0x1330 }, { 0x1332, 0x1333 }, { 0x1335, 0x1339 },
  { 0x133B, 0x1344 }, { 0x1347, 0x1348 }, { 0x134B, 0x134D }, { 0x1350, 0x1350 }, { 0x1357, 0x1357 },
  { 0x135D, 0x1363 }, { 0x1366, 0x136C }, { 0x1370, 0x1374 }, { 0x1380, 0x1389 }, { 0x138B, 0x138B },
  { 0x138E, 0x138E }, { 0x1390, 0x13B5 }, { 0x13B7, 0x13C0 }, { 0x13C2, 0x13C2 }, { 0x13C5, 0x13C5 },
  { 0x13C7, 0x13CA }, { 0x13CC, 0x13D5 }, { 0x13D7, 0x13D8 }, { 0x13E1, 0x13E2 }, { 0x1400, 0x145B },
  { 0x145D, 0x1461 }, { 0x1480, 0x14C7 }, { 0x14D0, 0x14D9 }, { 0x1580, 0x15B5 }, { 0x15B8, 0x15DD },
  { 0x1600, 0x1644 }, { 0x1650, 0x1659 }, { 0x1660, 0x166C }, { 0x1680, 0x16B9 }, { 0x16C0, 0x16C9 },
  { 0x16D0, 0x16E3 }, { 0x1700, 0x171A }, { 0x171D, 0x172B }, { 0x1730, 0x1746 }, { 0x1800, 0x183B },
  { 0x18A0, 0x18F2 }, { 0x18FF, 0x1906 }, { 0x1909, 0x1909 }, { 0x190C, 0x1913 }, { 0x1915, 0x1916 },
  { 0x1918, 0x1935 }, { 0x1937, 0x1938 }, { 0x193B, 0x1946 }, { 0x1950, 0x1959 }, { 0x19A0, 0x19A7 },
  { 0x19AA, 0x19D7 }, { 0x19DA, 0x19E4 }, { 0x1A00, 0x1A47 }, { 0x1A50, 0x1AA2 }, { 0x1AB0, 0x1AF8 },
  { 0x1B00, 0x1B09 }, { 0x1BC0, 0x1BE1 }, { 0x1BF0, 0x1BF9 }, { 0x1C00, 0x1C08 }, { 0x1C0A, 0x1C36 },
  { 0x1C38, 0x1C45 }, { 0x1C50, 0x1C6C }, { 0x1C70, 0x1C8F }, { 0x1C92, 0x1CA7 }, { 0x1CA9, 0x1CB6 },
  { 0x1D00, 0x1D06 }, { 0x1D08, 0x1D09 }, { 0x1D0B, 0x1D36 }, { 0x1D3A, 0x1D3A }, { 0x1D3C, 0x1D3D },
  { 0x1D3F, 0x1D47 }, { 0x1D50, 0x1D59 }, { 0x1D60, 0x1D65 }, { 0x1D67, 0x1D68 }, { 0x1D6A, 0x1D8E },
  { 0x1D90, 0x1D91 }, { 0x1D93, 0x1D98 }, { 0x1DA0, 0x1DA9 }, { 0x1EE0, 0x1EF8 }, { 0x1F00, 0x1F10 },
  { 0x1F12, 0x1F3A }, { 0x1F3E, 0x1F5A }, { 0x1FB0, 0x1FB0 }, { 0x1FC0, 0x1FF1 }, { 0x1FFF, 0x2399 },
  { 0x2400, 0x246E }, { 0x2470, 0x2474 }, { 0x2480, 0x2543 }, { 0x2F90, 0x2FF2 }, { 0x3000, 0x342F },
  { 0x3440, 0x3455 }, { 0x3460, 0x43FA }, { 0x4400, 0x4646 }, { 0x6100, 0x6139 }, { 0x6800, 0x6A38 },
  { 0x6A40, 0x6A5E }, { 0x6A60, 0x6A69 }, { 0x6A6E, 0x6ABE }, { 0x6AC0, 0x6AC9 }, { 0x6AD0, 0x6AED },
  { 0x6AF0, 0x6AF5 }, { 0x6B00, 0x6B45 }, { 0x6B50, 0x6B59 }, { 0x6B5B, 0x6B61 }, { 0x6B63, 0x6B77 },
  { 0x6B7D, 0x6B8F }, { 0x6D40, 0x6D79 }, { 0x6E40, 0x6E9A }, { 0x6F00, 0x6F4A }, { 0x6F4F, 0x6F87 },
  { 0x6F8F, 0x6F9F }, { 0x6FE0, 0x6FE4 }, { 0x6FF0, 0x6FF1 }, { 0x7000, 0x87F7 }, { 0x8800, 0x8CD5 },
  { 0x8CFF, 0x8D08 }, { 0xAFF0, 0xAFF3 }, { 0xAFF5, 0xAFFB }, { 0xAFFD, 0xAFFE }, { 0xB000, 0xB122 },
  { 0xB132, 0xB132 }, { 0xB150, 0xB152 }, { 0xB155, 0xB155 }, { 0xB164, 0xB167 }, { 0xB170, 0xB2FB },
  { 0xBC00, 0xBC6A }, { 0xBC70, 0xBC7C }, { 0xBC80, 0xBC88 }, { 0xBC90, 0xBC99 }, { 0xBC9C, 0xBC9F },
  { 0xCC00, 0xCCF9 }, { 0xCD00, 0xCEB3 }, { 0xCF00, 0xCF2D }, { 0xCF30, 0xCF46 }, { 0xCF50, 0xCFC3 },
  { 0xD000, 0xD0F5 }, { 0xD100, 0xD126 }, { 0xD129, 0xD172 }, { 0xD17B, 0xD1EA }, { 0xD200, 0xD245 },
  { 0xD2C0, 0xD2D3 }, { 0xD2E0, 0xD2F3 }, { 0xD300, 0xD356 }, { 0xD360, 0xD378 }, { 0xD400, 0xD454 },
  { 0xD456, 0xD49C }, { 0xD49E, 0xD49F }, { 0xD4A2, 0xD4A2 }, { 0xD4A5, 0xD4A6 }, { 0xD4A9, 0xD4AC },
  { 0xD4AE, 0xD4B9 }, { 0xD4BB, 0xD4BB }, { 0xD4BD, 0xD4C3 }, { 0xD4C5, 0xD505 }, { 0xD507, 0xD50A },
  { 0xD50D, 0xD514 }, { 0xD516, 0xD51C }, { 0xD51E, 0xD539 }, { 0xD53B, 0xD53E }, { 0xD540, 0xD544 },
  { 0xD546, 0xD546 }, { 0xD54A, 0xD550 }, { 0xD552, 0xD6A5 }, { 0xD6A8, 0xD7CB }, { 0xD7CE, 0xDA8B },
  { 0xDA9B, 0xDA9F }, { 0xDAA1, 0xDAAF }, { 0xDF00, 0xDF1E }, { 0xDF25, 0xDF2A }, { 0xE000, 0xE006 },
  { 0xE008, 0xE018 }, { 0xE01B, 0xE021 }, { 0xE023, 0xE024 }, { 0xE026, 0xE02A }, { 0xE030, 0xE06D },
  { 0xE08F, 0xE08F }, { 0xE100, 0xE12C }, { 0xE130, 0xE13D }, { 0xE140, 0xE149 }, { 0xE14E, 0xE14F },
  { 0xE290, 0xE2AE }, { 0xE2C0, 0xE2F9 }, { 0xE2FF, 0xE2FF }, { 0xE4D0, 0xE4F9 }, { 0xE5D0, 0xE5FA },
  { 0xE5FF, 0xE5FF }, { 0xE7E0, 0xE7E6 }, { 0xE7E8, 0xE7EB }, { 0xE7ED, 0xE7EE }, { 0xE7F0, 0xE7FE },
  { 0xE800, 0xE8C4 }, { 0xE8C7, 0xE8D6 }, { 0xE900, 0xE94B }, { 0xE950, 0xE959 }, { 0xE95E, 0xE95F },
  { 0xEC71, 0xECB4 }, { 0xED01, 0xED3D }, { 0xEE00, 0xEE03 }, { 0xEE05, 0xEE1F }, { 0xEE21, 0xEE22 },
  { 0xEE24, 0xEE24 }, { 0xEE27, 0xEE27 }, { 0xEE29, 0xEE32 }, { 0xEE34, 0xEE37 }, { 0xEE39, 0xEE39 },
  { 0xEE3B, 0xEE3B }, { 0xEE42, 0xEE42 }, { 0xEE47, 0xEE47 }, { 0xEE49, 0xEE49 }, { 0xEE4B, 0xEE4B },
  { 0xEE4D, 0xEE4F }, { 0xEE51, 0xEE52 }, { 0xEE54, 0xEE54 }, { 0xEE57, 0xEE57 }, { 0xEE59, 0xEE59 },
  { 0xEE5B, 0xEE5B }, { 0xEE5D, 0xEE5D }, { 0xEE5F, 0xEE5F }, { 0xEE61, 0xEE62 }, { 0xEE64, 0xEE64 },
  { 0xEE67, 0xEE6A }, { 0xEE6C, 0xEE72 }, { 0xEE74, 0xEE77 }, { 0xEE79, 0xEE7C }, { 0xEE7E, 0xEE7E },
  { 0xEE80, 0xEE89 }, { 0xEE8B, 0xEE9B }, { 0xEEA1, 0xEEA3 }, { 0xEEA5, 0xEEA9 }, { 0xEEAB, 0xEEBB },
  { 0xEEF0, 0xEEF1 }, { 0xF000, 0xF02B }, { 0xF030, 0xF093 }, { 0xF0A0, 0xF0AE }, { 0xF0B1, 0xF0BF },
  { 0xF0C1, 0xF0CF }, { 0xF0D1, 0xF0F5 }, { 0xF100, 0xF1AD }, { 0xF1E6, 0xF202 }, { 0xF210, 0xF23B },
  { 0xF240, 0xF248 }, { 0xF250, 0xF251 }, { 0xF260, 0xF265 }, { 0xF300, 0xF6D7 }, { 0xF6DC, 0xF6EC },
  { 0xF6F0, 0xF6FC }, { 0xF700, 0xF776 }, { 0xF77B, 0xF7D9 }, { 0xF7E0, 0xF7EB }, { 0xF7F0, 0xF7F0 },
  { 0xF800, 0xF80B }, { 0xF810, 0xF847 }, { 0xF850, 0xF859 }, { 0xF860, 0xF887 }, { 0xF890, 0xF8AD },
  { 0xF8B0, 0xF8BB }, { 0xF8C0, 0xF8C1 }, { 0xF900, 0xFA53 }, { 0xFA60, 0xFA6D }, { 0xFA70, 0xFA7C },
  { 0xFA80, 0xFA89 }, { 0xFA8F, 0xFAC6 }, { 0xFACE, 0xFADC }, { 0xFADF, 0xFAE9 }, { 0xFAF0, 0xFAF8 },
  { 0xFB00, 0xFB92 }, { 0xFB94, 0xFBF9 },
};
EFJSON_PRIVATE const efjsonUint32 EFJSON__GRAPH3[][2] = {
  { 0x20000, 0x2A6DF }, { 0x2A700, 0x2B739 }, { 0x2B740, 0x2B81D }, { 0x2B820, 0x2CEA1 }, { 0x2CEB0, 0x2EBE0 },
  { 0x2EBF0, 0x2EE5D }, { 0x2F800, 0x2FA1D }, { 0x30000, 0x3134A }, { 0x31350, 0x323AF }, { 0xE0100, 0xE01EF }
};
    #endif

EFJSON_PRIVATE int efjson__lookupTable16(efjsonUint16 u, const efjsonUint16 table[][2], unsigned size) {
  unsigned l = 0, r = size;
  while(l < r) {
    unsigned m = (l + r) >> 1u;
    efjson_assert(m >= l && m < r);
    if(u <= table[m][1]) r = m;
    else l = m + 1;
  }
  return l != size && u >= table[l][0];
}
EFJSON_PRIVATE int efjson__lookupTable32(efjsonUint32 u, const efjsonUint32 table[][2], unsigned size) {
  while(size-- != 0)
    if(u >= table[size][0] && u <= table[size][1]) return 1;
  return 0;
}

EFJSON_UAPI int efjson_isWhitespace(efjsonUint32 u, int fitJson5) {
  if(u == 0x20 || u == 0x09 || u == 0x0A || u == 0x0D) return 1;
  if(ul_unlikely(fitJson5)) {
    unsigned idx = sizeof(EFJSON__EXTRA_WHITESPACE) / sizeof(efjsonUint16);
    while(idx-- != 0)
      if(u == EFJSON__EXTRA_WHITESPACE[idx]) return 1;
  }
  return 0;
}
EFJSON_UAPI int efjson_isIdentifierStart(efjsonUint32 u) {
  if(ul_likely(u <= 0xFFFFu))
    return efjson__lookupTable16(
      efjson_cast(efjsonUint16, u), EFJSON__IDENTIFIER_START1,
      sizeof(EFJSON__IDENTIFIER_START1) / sizeof(EFJSON__IDENTIFIER_START1[0])
    );
  else if(ul_likely(u <= 0x1FFFFu))
    return efjson__lookupTable16(
      efjson_cast(efjsonUint16, u - 0x10000u), EFJSON__IDENTIFIER_START2,
      sizeof(EFJSON__IDENTIFIER_START2) / sizeof(EFJSON__IDENTIFIER_START2[0])
    );
  else
    return efjson__lookupTable32(
      efjson_cast(efjsonUint32, u), EFJSON__IDENTIFIER_START3,
      sizeof(EFJSON__IDENTIFIER_START3) / sizeof(EFJSON__IDENTIFIER_START3[0])
    );
}
EFJSON_UAPI int efjson_isIdentifierNext(efjsonUint32 u) {
  if(ul_likely(u <= 0xFFFFu)) {
    return efjson__lookupTable16(
             efjson_cast(efjsonUint16, u), EFJSON__IDENTIFIER_START1,
             sizeof(EFJSON__IDENTIFIER_START1) / sizeof(EFJSON__IDENTIFIER_START1[0])
           )
           || efjson__lookupTable16(
             efjson_cast(efjsonUint16, u), EFJSON__IDENTIFIER_NEXT_DELTA1,
             sizeof(EFJSON__IDENTIFIER_NEXT_DELTA1) / sizeof(EFJSON__IDENTIFIER_NEXT_DELTA1[0])
           );
  } else if(ul_likely(u <= 0x1FFFFu)) {
    return efjson__lookupTable16(
             efjson_cast(efjsonUint16, u - 0x10000u), EFJSON__IDENTIFIER_START2,
             sizeof(EFJSON__IDENTIFIER_START2) / sizeof(EFJSON__IDENTIFIER_START2[0])
           )
           || efjson__lookupTable16(
             efjson_cast(efjsonUint16, u - 0x10000u), EFJSON__IDENTIFIER_NEXT_DELTA2,
             sizeof(EFJSON__IDENTIFIER_NEXT_DELTA2) / sizeof(EFJSON__IDENTIFIER_NEXT_DELTA2[0])
           );
  } else {
    return efjson__lookupTable32(
             u, EFJSON__IDENTIFIER_START3, sizeof(EFJSON__IDENTIFIER_START3) / sizeof(EFJSON__IDENTIFIER_START3[0])
           )
           || efjson__lookupTable32(
             u, EFJSON__IDENTIFIER_NEXT_DELTA3,
             sizeof(EFJSON__IDENTIFIER_NEXT_DELTA3) / sizeof(EFJSON__IDENTIFIER_NEXT_DELTA3[0])
           );
  }
}
    #if EFJSON_CONF_EXPOSE_UNICODE
EFJSON_UAPI int efjson_isGraph(efjsonUint32 u) {
  if(ul_likely(u <= 0xFFFFu))
    return efjson__lookupTable16(
      efjson_cast(efjsonUint16, u), EFJSON__GRAPH1, sizeof(EFJSON__GRAPH1) / sizeof(EFJSON__GRAPH1[0])
    );
  else if(ul_likely(u <= 0x1FFFFu))
    return efjson__lookupTable16(
      efjson_cast(efjsonUint16, u - 0x10000u), EFJSON__GRAPH2, sizeof(EFJSON__GRAPH2) / sizeof(EFJSON__GRAPH2[0])
    );
  else
    return efjson__lookupTable32(
      efjson_cast(efjsonUint32, u), EFJSON__GRAPH3, sizeof(EFJSON__GRAPH3) / sizeof(EFJSON__GRAPH3[0])
    );
}
    #endif
  #else
EFJSON_UAPI int efjson_isWhitespace(efjsonUint32 u, int fitJson5) {
  (void)fitJson5;
  return u == 0x20 || u == 0x09 || u == 0x0A || u == 0x0D;
}
EFJSON_UAPI int efjson_isIdentifierStart(efjsonUint32 u) {
  return u == 0x5F || u == 0x24 || (u >= 0x61 && u <= 0x7A) || (u >= 0x41 && u <= 0x5A);
}
EFJSON_UAPI int efjson_isIdentifierNext(efjsonUint32 u) {
  return efjson_isIdentifierStart(u) || (u >= 0x30 && u <= 0x39);
}
    #if EFJSON_CONF_EXPOSE_UNICODE
EFJSON_UAPI int efjson_isGraph(efjsonUint32 u) {
  return u >= 0x20 && u < 0x7F;
}
    #endif
  #endif


EFJSON_PRIVATE int efjson__isNextLine(efjsonUint32 u) {
  return u == 0x0A /* '\n' */ || u == 0x0D /* '\r' */ || u == 0x2028 || u == 0x2029;
}
EFJSON_PRIVATE int efjson__isNumberSeparator(efjsonUint32 u, int fitJson5) {
  return efjson_isWhitespace(u, fitJson5) || u == 0x00
         || u == 0x2C /* ',' */ || u == 0x5D /* ']' */ || u == 0x7D /* '}' */ || u == 0x2F /* '/' */;
}
EFJSON_PRIVATE int efjson__isHexDigit(efjsonUint32 u) {
  return (u >= 0x30 && u <= 0x39) || (u >= 0x41 && u <= 0x46) || (u >= 0x61 && u <= 0x66);
}
  #define efjson__hexDigit(u) ((u) <= 0x39u ? (u) - 0x30u : ((u) & 0xF) + 9u)
EFJSON_PRIVATE int efjson__isControl(efjsonUint32 u) {
  return u <= 0x1F || u == 0x7F;
}
  #define efjson__isUtf16Surrogate(c) ((c) >= 0xD800u && (c) <= 0xDFFFu)


  /******************************
   * UTF-8 / UTF-16
   ******************************/


  #if EFJSON_CONF_UTF_ENCODER
EFJSON_PUBLIC size_t efjsonUtf8Decoder_sizeof(void) {
  return sizeof(efjsonUtf8Decoder);
}
EFJSON_PUBLIC efjsonUtf8Decoder* efjsonUtf8Decoder_new(void) {
  return efjson_reptr(efjsonUtf8Decoder*, calloc(1, sizeof(efjsonUtf8Decoder)));
}
EFJSON_PUBLIC void efjsonUtf8Decoder_destroy(efjsonUtf8Decoder* decoder) {
  free(decoder);
}
EFJSON_PUBLIC void efjsonUtf8Decoder_init(efjsonUtf8Decoder* decoder) {
  memset(decoder, 0, sizeof(efjsonUtf8Decoder));
}
EFJSON_PUBLIC int efjsonUtf8Decoder_feed(efjsonUtf8Decoder* decoder, efjsonUint32* result, efjsonUint8 c) {
  if(decoder->rest == 0) {
    if(c <= 0x7F) {
      *result = efjson_cast(efjsonUint32, c);
      return 1;
    } else if(ul_unlikely(c < 0xC2)) {
      return -1;
    } else if(c <= 0xDF) {
      decoder->total = decoder->rest = 1;
      decoder->code = efjson_cast(efjsonUint32, c & 0x1F);
      return 0;
    } else if(c <= 0xEF) {
      decoder->total = decoder->rest = 2;
      decoder->code = efjson_cast(efjsonUint32, c & 0xF);
      return 0;
    } else if(ul_likely(c <= 0xF4)) {
      decoder->total = decoder->rest = 3;
      decoder->code = efjson_cast(efjsonUint32, c & 0x7);
      return 0;
    } else {
      return -1;
    }
  }

  if(ul_unlikely((c & 0xC0) != 0x80)) return -1;
  decoder->code = (decoder->code << 6) | (c & 0x3F);
  if(--decoder->rest != 0) return 0;

  if(decoder->code <= 0x7F) {
    if(ul_unlikely(decoder->total != 0)) return -1;
  } else if(decoder->code <= 0x7FF) {
    if(ul_unlikely(decoder->total != 1)) return -1;
  } else if(ul_likely(decoder->code <= 0xFFFF)) {
    if(ul_unlikely(decoder->total != 2)) return -1;
  } else if(ul_likely(decoder->code <= 0x10FFFF)) {
    if(ul_unlikely(decoder->total != 3)) return -1;
  } else {
    return -1;
  }
  *result = decoder->code;
  return 1;
}
EFJSON_PUBLIC int efjson_EncodeUtf8(efjsonUint8* p, efjsonUint32 u) {
  efjsonUint8* q = p;
  if(u <= 0x7Fu) {
    *q++ = efjson_cast(efjsonUint8, u);
  } else {
    if(u <= 0x7FFu) {
      *q++ = efjson_cast(efjsonUint8, (u >> 6) | 0xC0);
    } else {
      if(u <= 0xFFFFu) {
        *q++ = efjson_cast(efjsonUint8, (u >> 12) | 0xE0);
      } else {
        if(ul_likely(u <= 0x10FFFFu)) {
          *q++ = efjson_cast(efjsonUint8, (u >> 18) | 0xF0);
        } else {
          return -1;
        }
        *q++ = efjson_cast(efjsonUint8, ((u >> 12) & 0x3F) | 0x80);
      }
      *q++ = efjson_cast(efjsonUint8, ((u >> 6) & 0x3F) | 0x80);
    }
    *q++ = efjson_cast(efjsonUint8, (u & 0x3F) | 0x80);
  }
  return efjson_cast(int, q - p);
}


EFJSON_PUBLIC size_t efjsonUtf16Decoder_sizeof(void) {
  return sizeof(efjsonUtf16Decoder);
}
EFJSON_PUBLIC efjsonUtf16Decoder* efjsonUtf16Decoder_new(void) {
  return efjson_reptr(efjsonUtf16Decoder*, calloc(1, sizeof(efjsonUtf16Decoder)));
}
EFJSON_PUBLIC void efjsonUtf16Decoder_destroy(efjsonUtf16Decoder* decoder) {
  free(decoder);
}
EFJSON_PUBLIC void efjsonUtf16Decoder_init(efjsonUtf16Decoder* decoder) {
  memset(decoder, 0, sizeof(efjsonUtf16Decoder));
}
EFJSON_PUBLIC int efjsonUtf16Decoder_feed(efjsonUtf16Decoder* decoder, efjsonUint32* result, efjsonUint16 c) {
  if(decoder->first != 0) {
    if(ul_likely(0xDC00u <= c && c <= 0xDFFFu)) {
      *result = (efjson_cast(efjsonUint32, decoder->first & 0x3FFu) << 10 | (c & 0x3FFu)) + 0x10000u;
      return 1;
    }
    return -1;
  }
  if(0xD800u <= c && c <= 0xDBFFu) {
    decoder->first = c;
    return 0;
  }
  if(ul_unlikely(0xDC00u <= c && c <= 0xDFFFu)) return -1;
  *result = efjson_cast(efjsonUint32, c);
  return 1;
}
EFJSON_PUBLIC int efjson_EncodeUtf16(efjsonUint16* p, efjsonUint32 u) {
  if(ul_unlikely(u >= 0xD800 && u <= 0xDFFF)) return -1;
  if(u < 0x10000u) {
    *p = efjson_cast(efjsonUint16, u);
    return 1;
  } else if(ul_likely(u <= 0x10FFFFu)) {
    u -= 0x10000u;
    p[0] = efjson_cast(efjsonUint16, 0xD800 | (u >> 10));
    p[1] = efjson_cast(efjsonUint16, 0xDC00 | (u & 0x3FFu));
    return 2;
  }
  return -1;
}
  #endif /* EFJSON_CONF_UTF_ENCODER */


/******************************
 * Internal state
 ******************************/


enum {
  efjsonVal__EMPTY,
  efjsonVal__NULL,
  efjsonVal__TRUE,
  efjsonVal__FALSE,
  efjsonVal__STRING,
  efjsonVal__STRING_ESCAPE,
  efjsonVal__STRING_UNICODE,
  #if EFJSON_CONF_COMBINE_ESCAPED_SURROGATE
  efjsonVal__STRING_UNICODE_NEXT,
  #endif
  efjsonVal__NUMBER,
  efjsonVal__NUMBER_FRACTION,
  efjsonVal__NUMBER_EXPONENT,
  /* JSON5 */
  efjsonVal__STRING_MULTILINE_CR, /* used to check \r\n */
  efjsonVal__STRING_ESCAPE_HEX,   /* used to check \xNN */
  efjsonVal__NUMBER_INFINITY,     /* used to check "Infinity" */
  efjsonVal__NUMBER_NAN,          /* used to check "NaN" */
  efjsonVal__NUMBER_HEX,          /* used to check hexadecimal number */
  efjsonVal__NUMBER_OCT,          /* used to check octal number */
  efjsonVal__NUMBER_BIN,          /* used to check binary number */

  efjsonVal__COMMENT_MAY_START,          /* used to check comment */
  efjsonVal__SINGLE_LINE_COMMENT,        /* used to check single line comment */
  efjsonVal__MULTI_LINE_COMMENT,         /* used to check multi-line comment */
  efjsonVal__MULTI_LINE_COMMENT_MAY_END, /* used to check multi-line comment */

  efjsonVal__IDENTIFIER,       /* used to check identifier key */
  efjsonVal__IDENTIFIER_ESCAPE /* used to check identifier key */
};

enum {
  efjsonLoc__ROOT_START,
  efjsonLoc__KEY_FIRST_START, /* used to check trailing comma */
  efjsonLoc__KEY_START,
  efjsonLoc__VALUE_START,
  efjsonLoc__ELEMENT_FIRST_START, /* used to check trailing comma */
  efjsonLoc__ELEMENT_START,

  efjsonLoc__ROOT_END,
  efjsonLoc__KEY_END,
  efjsonLoc__VALUE_END,
  efjsonLoc__ELEMENT_END,

  efjsonLoc__EOF
};
EFJSON_PRIVATE const efjsonUint8 efjson__NEXT_LOCATION_TABLE[] = {
  efjsonLoc__ROOT_END,  efjsonLoc__KEY_END,     efjsonLoc__KEY_END,
  efjsonLoc__VALUE_END, efjsonLoc__ELEMENT_END, efjsonLoc__ELEMENT_END,
};
  #define efjson__nextLocation(loc) \
    efjson_condexpr((loc) <= efjsonLoc__ELEMENT_START, efjson__NEXT_LOCATION_TABLE[(loc)])

EFJSON_PRIVATE const efjsonUint8 efjson__LOCATION_TABLE[] = {
  efjsonLocation_ROOT,    efjsonLocation_KEY,     efjsonLocation_KEY,  efjsonLocation_VALUE,
  efjsonLocation_ELEMENT, efjsonLocation_ELEMENT, efjsonLocation_ROOT, efjsonLocation_KEY,
  efjsonLocation_VALUE,   efjsonLocation_ELEMENT, efjsonLocation_ROOT
};
  #define efjson__transformLocation(loc) (efjson__LOCATION_TABLE[(loc)])


enum {
  efjsonFlag__MeetCr = 0x1,
  efjsonFlag__SingleQuote = 0x2
};


EFJSON_PRIVATE const efjsonUint8 efjson__LITERAL_NULL[] = { 0x6E, 0x75, 0x6C, 0x6C } /* "null" */;
EFJSON_PRIVATE const efjsonUint8 efjson__LITERAL_TRUE[] = { 0x74, 0x72, 0x75, 0x65 } /* "true" */;
EFJSON_PRIVATE const efjsonUint8 efjson__LITERAL_FALSE[] = { 0x66, 0x61, 0x6C, 0x73, 0x65 } /* "false" */;
EFJSON_PRIVATE const efjsonUint8 efjson__LITERAL_INFINITY[] = {
  0x49, 0x6E, 0x66, 0x69, 0x6E, 0x69, 0x74, 0x79
} /* "Infinity" */;
EFJSON_PRIVATE const efjsonUint8 efjson__LITERAL_NAN[] = { 0x4E, 0x61, 0x4E } /* "NaN" */;


/******************************
 * Parsing
 ******************************/

enum {
  /** accept sign, not yet accept number */
  efjsonNumberState__ONLY_SIGN = 0xFF,
  /** already accept +0/-0 */
  efjsonNumberState__ZERO = 0x0,
  /** already accept non-leading 0 number */
  efjsonNumberState__NON_LEADING_ZERO = 0x1
};

enum {
  /** not yet accept any */
  efjsonNumberExponent__NOT_YET = 0,
  /** already accept sign, not accept digits */
  efjsonNumberExponent__AFTER_SIGN = 1,
  /** already accept digits */
  efjsonNumberExponent__AFTER_DIGIT = 2
};

  #if EFJSON_CONF_COMPRESS_STACK
    #define efjson__stackLen(n) efjson_cast(efjsonStackLength, (n) >> 3)
    #define efjson__bitshl(v, n) efjson_cast(efjsonUint8, (v) << (n))

    #define efjson___pushArray(parser, loc)                                                         \
      efjson_condexpr(                                                                              \
        ((parser)->len == 0 && (loc) == efjsonLoc__ROOT_START) || (loc) == efjsonLoc__ELEMENT_START \
          || (loc) == efjsonLoc__ELEMENT_FIRST_START,                                               \
        (parser)->stack[(parser)->len >> 3] |= efjson__bitshl(1, (parser)->len & 7)                 \
      )
    #define efjson___pushObject(parser, loc) \
      ((parser)->stack[(parser)->len >> 3] &= efjson_cast(efjsonUint8, ~efjson__bitshl(1, (parser)->len & 7)))

    #define efjson__push(parser, loc)                                                                                \
      ((void)((loc) == efjsonLoc__VALUE_START ? efjson___pushObject(parser, loc) : efjson___pushArray(parser, loc)), \
       ++(parser)->len)
    #define efjson__last(parser)                                                                      \
      (ul_unlikely((parser)->len == 0)                                                                \
         ? efjsonLoc__ROOT_END                                                                        \
         : (((parser)->stack[(parser)->len >> 3] >> ((parser)->len & 7)) & 1 ? efjsonLoc__ELEMENT_END \
                                                                             : efjsonLoc__VALUE_END))
  #else
    #define efjson__stackLen(n) (n)
    #define efjson__push(parser, loc) ((parser)->stack[(parser)->len++] = loc)
    #define efjson__last(parser) efjson__transformLocation((parser)->stack[(parser)->len])
  #endif


  /**
   * `substate` and `escape` in `JsonStreamParser`
   *   `STRING_UNICODE`: <substate: 0..4> the index of the Unicode sequence
   *                     <escape> escaped character
   *
   *   `NUMBER`: <substate> `efjsonNumberState__*`
   *
   *   `NUMBER_FRACTION`: <substate: 0|1> whether already accept digits
   *
   *   `NUMBER_EXPONENT`: <substate> `efjsonNumberExponent__`
   *
   *   `NULL`/`TRUE`/`FALSE`: <substate: 0..5> current index of string
   *
   *   `STRING_ESCAPE_HEX`: <substate: 0..2> the index of the Hex sequence
   *                        <escape> escaped character
   *
   *   `NUMBER_NAN`|`NUMBER_INFINITY`: <substate: 0..8> current index of string
   *
   *   `NUMBER_HEX`|`NUMBER_OCT`|`NUMBER_BIN`: <substate: 0|1> whether already accept digits
   *
   *   `IDENTIFIER_ESCAPE`: <substate: 0..5> the index of the Unicode sequence (includes 'u' prefix)
   *                        <escape> escaped character
   */

  #if !(EFJSON_CONF_FIXED_STACK > 0)
EFJSON_PRIVATE int efjsonStreamParser__enlarge(efjsonStreamParser* parser) {
  efjsonStackLength newCap = efjson_cast(efjsonStackLength, parser->cap + (parser->cap >> 1) + 1);
  efjsonUint8* newStack;
  if(ul_unlikely(newCap <= efjson__stackLen(parser->len))) { /* avoid `efjsonStackLength` overflow */
    newCap = efjson__stackLen(efjson_umax(efjsonStackLength));
    if(ul_unlikely(newCap <= efjson__stackLen(parser->len))) return efjsonError_TOO_MANY_RECURSIONS;
  }
    #if EFJSON_CONF_CHECK_SIZET_OVERFLOW
  if(ul_unlikely(newCap > efjson_umax(size_t))) /* avoid `size_t` overflow */
    return efjsonError_ALLOC_FAILED;
    #endif
  newStack = efjson_reptr(efjsonUint8*, realloc(parser->stack, efjson_cast(size_t, newCap)));
  if(ul_unlikely(!newStack)) return efjsonError_ALLOC_FAILED;
  parser->stack = newStack;
  parser->cap = newCap;
  return 0;
}
  #endif

EFJSON_PRIVATE void efjsonStreamParser__handleNumberSeparator(
  efjsonStreamParser* parser, efjsonUint32 u, efjsonToken* token
) {
  parser->state = efjsonVal__EMPTY;
  parser->location = efjson__nextLocation(parser->location);
  if(u == 0x00) {
    if(parser->location == efjsonLoc__ROOT_START || parser->location == efjsonLoc__ROOT_END) {
      token->type = efjsonType_EOF;
      parser->location = efjsonLoc__EOF;
    } else {
      token->extra = efjsonError_EOF;
    }
  } else if(u == 0x7D /* '}' */) {
    if(parser->location == efjsonLoc__KEY_FIRST_START || parser->location == efjsonLoc__VALUE_END) {
      --parser->len;
      parser->state = efjsonVal__EMPTY;
      parser->location = efjson__last(parser);
      token->location = efjson__transformLocation(parser->location);
      token->type = efjsonType_OBJECT_END;
    } else if(parser->location == efjsonLoc__KEY_START) {
      if(parser->option & efjsonOption_TRAILING_COMMA_IN_OBJECT) {
        --parser->len;
        parser->state = efjsonVal__EMPTY;
        parser->location = efjson__last(parser);
        token->location = efjson__transformLocation(parser->location);
        token->type = efjsonType_OBJECT_END;
      } else {
        token->extra = efjsonError_COMMA_IN_EMPTY_OBJECT;
      }
    } else {
      token->extra = efjsonError_WRONG_BRACKET;
    }
  } else if(u == 0x5D /* ']' */) {
    if(parser->location == efjsonLoc__ELEMENT_FIRST_START || parser->location == efjsonLoc__ELEMENT_END) {
      --parser->len;
      parser->state = efjsonVal__EMPTY;
      parser->location = efjson__last(parser);
      token->location = efjson__transformLocation(parser->location);
      token->type = efjsonType_ARRAY_END;
    } else if(parser->location == efjsonLoc__ELEMENT_START) {
      if(parser->option & efjsonOption_TRAILING_COMMA_IN_ARRAY) {
        --parser->len;
        parser->state = efjsonVal__EMPTY;
        parser->location = efjson__last(parser);
        token->location = efjson__transformLocation(parser->location);
        token->type = efjsonType_ARRAY_END;
      } else {
        token->extra = efjsonError_COMMA_IN_EMPTY_ARRAY;
      }
    } else {
      token->extra = efjsonError_WRONG_BRACKET;
    }
  } else if(u == 0x2C /* ',' */) {
    if(parser->location == efjsonLoc__VALUE_END) {
      parser->location = efjsonLoc__KEY_START;
      token->location = efjsonLocation_OBJECT;
      token->type = efjsonType_OBJECT_NEXT;
    } else if(parser->location == efjsonLoc__ELEMENT_END) {
      parser->location = efjsonLoc__ELEMENT_START;
      token->location = efjsonLocation_OBJECT;
      token->type = efjsonType_ARRAY_NEXT;
    } else if(parser->location == efjsonLoc__ELEMENT_FIRST_START) {
      token->extra = efjsonError_COMMA_IN_EMPTY_ARRAY;
    } else if(parser->location == efjsonLoc__ELEMENT_START) {
      token->extra = efjsonError_TRAILING_COMMA_FORBIDDEN;
    } else if(parser->location == efjsonLoc__VALUE_START) {
      token->extra = efjsonError_EMPTY_VALUE_IN_OBJECT;
    } else {
      token->extra = efjsonError_UNEXPECTED;
    }
  } else if(u == 0x2F /* '/' */) {
    if(parser->option & (efjsonOption_SINGLE_LINE_COMMENT | efjsonOption_MULTI_LINE_COMMENT)) {
      parser->state = efjsonVal__COMMENT_MAY_START;
      token->type = efjsonType_COMMENT_MAY_START;
    } else token->extra = efjsonError_COMMENT_FORBIDDEN;
  } else {
    token->location = efjson__transformLocation(parser->location);
    token->type = efjsonType_WHITESPACE;
  }
}
EFJSON_PRIVATE void efjsonStreamParser__handleEmpty(efjsonStreamParser* parser, efjsonUint32 u, efjsonToken* token) {
  if(efjson_isWhitespace(u, (parser->option & efjsonOption_JSON5_WHITESPACE) != 0)) {
    token->type = efjsonType_WHITESPACE;
  } else if(u == 0x00) {
    if(ul_likely(parser->location == efjsonLoc__ROOT_START || parser->location == efjsonLoc__ROOT_END)) {
      parser->location = efjsonLoc__EOF;
      token->type = efjsonType_EOF;
    } else {
      token->extra = efjsonError_EOF;
    }
  } else if(u == 0x2F /* '/' */) {
    if(parser->option & (efjsonOption_SINGLE_LINE_COMMENT | efjsonOption_MULTI_LINE_COMMENT)) {
      parser->state = efjsonVal__COMMENT_MAY_START;
      token->type = efjsonType_COMMENT_MAY_START;
    } else {
      token->extra = efjsonError_COMMENT_FORBIDDEN;
    }
  } else if(ul_unlikely(parser->location == efjsonLoc__ROOT_END)) {
    token->extra = efjsonError_NONWHITESPACE_AFTER_END;
  } else if(u == 0x22 /* '"' */) {
    parser->state = efjsonVal__STRING;
    parser->flag &= ~efjsonFlag__SingleQuote;
    token->type = efjsonType_STRING_START;
  } else if(u == 0x27 /* '\'' */) {
    if(ul_unlikely(parser->option & efjsonOption_SINGLE_QUOTE)) {
      parser->state = efjsonVal__STRING;
      parser->flag |= efjsonFlag__SingleQuote;
      token->type = efjsonType_STRING_START;
    } else {
      token->extra = efjsonError_SINGLE_QUOTE_FORBIDDEN;
    }
  } else {
    if(parser->location == efjsonLoc__KEY_FIRST_START || parser->location == efjsonLoc__KEY_START) {
      if(parser->option & efjsonOption_IDENTIFIER_KEY) {
        if(efjson_isIdentifierStart(u)) {
          parser->state = efjsonVal__IDENTIFIER;
          token->type = efjsonType_IDENTIFIER_NORMAL;
          return;
        } else if(u == 0x5C /* '\\' */) {
          parser->state = efjsonVal__IDENTIFIER_ESCAPE;
          parser->substate = 0;
          token->type = efjsonType_IDENTIFIER_ESCAPE_START;
          return;
        }
      }
      if(ul_unlikely(u != 0x7D /* '}' */)) {
        token->extra = efjsonError_BAD_PROPERTY_NAME_IN_OBJECT;
        return;
      }
    }

    if(u == 0x3A /* ':' */) {
      if(ul_likely(parser->location == efjsonLoc__KEY_END)) {
        parser->location = efjsonLoc__VALUE_START;
        token->location = efjsonLocation_OBJECT;
        token->type = efjsonType_OBJECT_VALUE_START;
      } else if(parser->location == efjsonLoc__VALUE_START) token->extra = efjsonError_REPEATED_COLON;
      else token->extra = efjsonError_WRONG_COLON;
    } else if(ul_unlikely(parser->location == efjsonLoc__KEY_END)) token->extra = efjsonError_EXPECTED_COLON;
    else if(u == 0x5D /* ']' */) {
      if(parser->location == efjsonLoc__ELEMENT_FIRST_START || parser->location == efjsonLoc__ELEMENT_END) {
        --parser->len;
        parser->location = efjson__last(parser);
        token->location = efjson__transformLocation(parser->location);
        token->type = efjsonType_ARRAY_END;
      } else if(parser->location == efjsonLoc__ELEMENT_START) {
        if(parser->option & efjsonOption_TRAILING_COMMA_IN_ARRAY) {
          --parser->len;
          parser->location = efjson__last(parser);
          token->type = efjsonType_ARRAY_END;
        } else {
          token->extra = efjsonError_COMMA_IN_EMPTY_ARRAY;
        }
      } else {
        token->extra = efjsonError_WRONG_BRACKET;
      }
    } else if(u == 0x7D /* '}' */) {
      if(parser->location == efjsonLoc__KEY_FIRST_START || parser->location == efjsonLoc__VALUE_END) {
        --parser->len;
        parser->location = efjson__last(parser);
        token->location = efjson__transformLocation(parser->location);
        token->type = efjsonType_OBJECT_END;
      } else if(ul_likely(parser->location == efjsonLoc__KEY_START)) {
        if(parser->option & efjsonOption_TRAILING_COMMA_IN_OBJECT) {
          --parser->len;
          parser->location = efjson__last(parser);
          token->location = efjson__transformLocation(parser->location);
          token->type = efjsonType_OBJECT_END;
        } else {
          token->extra = efjsonError_COMMA_IN_EMPTY_OBJECT;
        }
      } else {
        token->extra = efjsonError_WRONG_BRACKET;
      }
    } else if(u == 0x2C /* ',' */) {
      if(parser->location == efjsonLoc__VALUE_END) {
        parser->location = efjsonLoc__KEY_START;
        token->location = efjsonLocation_OBJECT;
        token->type = efjsonType_OBJECT_NEXT;
      } else if(ul_likely(parser->location == efjsonLoc__ELEMENT_END)) {
        parser->location = efjsonLoc__ELEMENT_START;
        token->location = efjsonLocation_ARRAY;
        token->type = efjsonType_ARRAY_NEXT;
      } else if(parser->location == efjsonLoc__ELEMENT_FIRST_START) {
        token->extra = efjsonError_COMMA_IN_EMPTY_ARRAY;
      } else if(parser->location == efjsonLoc__ELEMENT_START) {
        token->extra = efjsonError_TRAILING_COMMA_FORBIDDEN;
      } else if(parser->location == efjsonLoc__VALUE_START) {
        token->extra = efjsonError_EMPTY_VALUE_IN_OBJECT;
      } else {
        token->extra = efjsonError_UNEXPECTED;
      }
    } else if(parser->location == efjsonLoc__ELEMENT_END || parser->location == efjsonLoc__VALUE_END) {
      token->extra = efjsonError_UNEXPECTED;
    } else switch(u) {
      case 0x5B /* '[' */:
  #if EFJSON_CONF_FIXED_STACK > 0
        if(ul_unlikely(efjson__stackLen(parser->len) == EFJSON_CONF_FIXED_STACK)) {
          token->extra = efjsonError_TOO_MANY_RECURSIONS;
          return;
        }
  #else
        if(ul_unlikely(efjson__stackLen(parser->len) == parser->cap)) {
          token->extra = efjsonStreamParser__enlarge(parser);
          if(ul_unlikely(token->extra != 0)) return;
        }
  #endif
        efjson__push(parser, parser->location);
        parser->location = efjsonLoc__ELEMENT_FIRST_START;
        token->location = efjson__transformLocation(parser->location);
        token->type = efjsonType_ARRAY_START;
        break;
      case 0x7B /* '{' */:
  #if EFJSON_CONF_FIXED_STACK > 0
        if(ul_unlikely(efjson__stackLen(parser->len) == EFJSON_CONF_FIXED_STACK)) {
          token->extra = efjsonError_TOO_MANY_RECURSIONS;
          return;
        }
  #else
        if(ul_unlikely(efjson__stackLen(parser->len) == parser->cap)) {
          token->extra = efjsonStreamParser__enlarge(parser);
          if(ul_unlikely(token->extra != 0)) return;
        }
  #endif
        efjson__push(parser, parser->location);
        parser->location = efjsonLoc__KEY_FIRST_START;
        token->type = efjsonType_OBJECT_START;
        break;

      case 0x2B /* '+' */:
        if(!(parser->option & efjsonOption_POSITIVE_SIGN)) {
          token->extra = efjsonError_POSITIVE_SIGN_FORBIDDEN;
          break;
        }
        ul_fallthrough;
      case 0x2D /* '-' */:
        parser->state = efjsonVal__NUMBER;
        parser->substate = efjsonNumberState__ONLY_SIGN;
        token->type = efjsonType_NUMBER_INTEGER_SIGN;
        break;
      case 0x30 /* '0' */:
      case 0x31 /* '1' */:
      case 0x32 /* '2' */:
      case 0x33 /* '3' */:
      case 0x34 /* '4' */:
      case 0x35 /* '5' */:
      case 0x36 /* '6' */:
      case 0x37 /* '7' */:
      case 0x38 /* '8' */:
      case 0x39 /* '9' */:
        parser->state = efjsonVal__NUMBER;
        parser->substate = (u != '0');
        token->type = efjsonType_NUMBER_INTEGER_DIGIT;
        break;
      case 0x2E /* '.' */:
        if(parser->option & efjsonOption_EMPTY_INTEGER) {
          parser->state = efjsonVal__NUMBER_FRACTION;
          parser->substate = 0;
          token->type = efjsonType_NUMBER_FRACTION_START;
        } else token->extra = efjsonError_EMPTY_INTEGER_PART;
        break;
      case 0x4E /* 'N' */:
        if(parser->option & efjsonOption_NAN) {
          parser->state = efjsonVal__NUMBER_NAN;
          parser->substate = 1;
          token->type = efjsonType_NUMBER_NAN;
        } else token->extra = efjsonError_UNEXPECTED_IN_NUMBER;
        break;
      case 0x49 /* 'I' */:
        if(parser->option & efjsonOption_INFINITY) {
          parser->state = efjsonVal__NUMBER_INFINITY;
          parser->substate = 1;
          token->type = efjsonType_NUMBER_INFINITY;
        } else token->extra = efjsonError_UNEXPECTED_IN_NUMBER;
        break;

      case 0x6E /* 'n' */:
        parser->state = efjsonVal__NULL;
        parser->substate = 1;
        token->type = efjsonType_NULL;
        break;
      case 0x74 /* 't' */:
        parser->state = efjsonVal__TRUE;
        parser->substate = 1;
        token->type = efjsonType_TRUE;
        break;
      case 0x66 /* 'f' */:
        parser->state = efjsonVal__FALSE;
        parser->substate = 1;
        token->type = efjsonType_FALSE;
        break;

      default:
        token->extra = efjsonError_UNEXPECTED;
      }
  }
}
EFJSON_PRIVATE efjsonToken efjsonStreamParser__step(efjsonStreamParser* parser, efjsonUint32 u) {
  efjsonToken token = { /* .type = */ efjsonType_ERROR,
                        /* .location = */ 0,
                        /* .index = */ 0,
                        /* .done = */ 0,
                        /* .extra = */ 0 };
  token.location = efjson__transformLocation(parser->location);
  if(ul_unlikely(parser->location == efjsonLoc__EOF)) {
    token.extra = efjsonError_CONTENT_AFTER_EOF;
    return token;
  }
  switch(parser->state) {
  case efjsonVal__EMPTY:
    efjsonStreamParser__handleEmpty(parser, u, &token);
    break;
  case efjsonVal__NULL:
    if(ul_likely(u == efjson__LITERAL_NULL[parser->substate])) {
      token.type = efjsonType_NULL;
      token.index = parser->substate;
      if((token.done = (++parser->substate == 4))) {
        parser->state = efjsonVal__EMPTY;
        parser->location = efjson__nextLocation(parser->location);
      }
    } else token.extra = efjsonError_UNEXPECTED;
    break;
  case efjsonVal__FALSE:
    if(ul_likely(u == efjson__LITERAL_FALSE[parser->substate])) {
      token.type = efjsonType_FALSE;
      token.index = parser->substate;
      if((token.done = (++parser->substate == 5))) {
        parser->state = efjsonVal__EMPTY;
        parser->location = efjson__nextLocation(parser->location);
      }
    } else token.extra = efjsonError_UNEXPECTED;
    break;
  case efjsonVal__TRUE:
    if(ul_likely(u == efjson__LITERAL_TRUE[parser->substate])) {
      token.type = efjsonType_TRUE;
      token.index = parser->substate;
      if((token.done = (++parser->substate == 4))) {
        parser->state = efjsonVal__EMPTY;
        parser->location = efjson__nextLocation(parser->location);
      }
    } else token.extra = efjsonError_UNEXPECTED;
    break;
  case efjsonVal__NUMBER_INFINITY:
    if(ul_likely(u == efjson__LITERAL_INFINITY[parser->substate])) {
      token.type = efjsonType_NUMBER_INFINITY;
      token.index = parser->substate;
      if((token.done = (++parser->substate == 8))) {
        parser->state = efjsonVal__EMPTY;
        parser->location = efjson__nextLocation(parser->location);
      }
    } else token.extra = efjsonError_UNEXPECTED_IN_NUMBER;
    break;
  case efjsonVal__NUMBER_NAN:
    if(ul_likely(u == efjson__LITERAL_NAN[parser->substate])) {
      token.type = efjsonType_NUMBER_NAN;
      token.index = parser->substate;
      if((token.done = (++parser->substate == 3))) {
        parser->state = efjsonVal__EMPTY;
        parser->location = efjson__nextLocation(parser->location);
      }
    } else token.extra = efjsonError_UNEXPECTED_IN_NUMBER;
    break;

  case efjsonVal__STRING_MULTILINE_CR:
    if(ul_likely(u == 0x0A /* '\n' */)) {
      parser->state = efjsonVal__STRING;
      token.type = efjsonType_STRING_NEXT_LINE;
      break;
    }
    ul_fallthrough;
  case efjsonVal__STRING:
    if(u == (parser->flag & efjsonFlag__SingleQuote ? 0x27 /* '\'' */ : 0x22 /* '"' */)) {
      parser->location = efjson__nextLocation(parser->location);
      parser->state = efjsonVal__EMPTY;
      token.type = efjsonType_STRING_END;
    } else if(ul_likely(u == 0x5C /* '\\' */)) {
      parser->state = efjsonVal__STRING_ESCAPE;
      token.type = efjsonType_STRING_ESCAPE_START;
    } else if(ul_unlikely(u == 0x00)) token.extra = efjsonError_EOF;
    else if(ul_unlikely(efjson__isControl(u))) token.extra = efjsonError_CONTROL_CHARACTER_FORBIDDEN_IN_STRING;
  #if EFJSON_CONF_CHECK_INPUT_UTF
    else if(ul_unlikely(efjson__isUtf16Surrogate(u) || u > 0x10FFFFu)) token.extra = efjsonError_INVALID_INPUT_UTF;
  #endif
    else token.type = efjsonType_STRING_NORMAL;
    break;
  case efjsonVal__STRING_ESCAPE:
    if(u == 0x75 /* 'u' */) {
      parser->state = efjsonVal__STRING_UNICODE;
      parser->substate = 0;
      parser->escape = 0;
      token.type = efjsonType_STRING_ESCAPE_UNICODE_START;
      break;
    }
    {
      efjsonUint8 u2 = 0xFF;
      switch(u) {
      case 0x22 /* '"' */:
        u2 = 0x22 /* '"' */;
        break;
      case 0x5C /* '\\' */:
        u2 = 0x5C /* '\\' */;
        break;
      case 0x2F /* '/' */:
        u2 = 0x2F /* '/' */;
        break;
      case 0x62 /* 'b' */:
        u2 = 0x08 /* '\b' */;
        break;
      case 0x66 /* 'f' */:
        u2 = 0x0C /* '\f' */;
        break;
      case 0x6E /* 'n' */:
        u2 = 0x0A /* '\n' */;
        break;
      case 0x72 /* 'r' */:
        u2 = 0x0D /* '\r' */;
        break;
      case 0x74 /* 't' */:
        u2 = 0x09 /* '\t' */;
        break;
      }
      if(parser->option & efjsonOption_JSON5_STRING_ESCAPE) switch(u) {
        case 0x27 /* '\'' */:
          u2 = 0x27 /* '\'' */;
          break;
        case 0x76 /* 'v' */:
          u2 = 0x0b /* '\v' */;
          break;
        case 0x30 /* '0' */:
          u2 = 0x00 /* '\0' */;
          break;
        }
      if(ul_likely(u2 != 0xFF)) {
        parser->state = efjsonVal__STRING;
        token.type = efjsonType_STRING_ESCAPE;
        token.done = 1;
        token.extra = u2;
        break;
      }
    }
    if((parser->option & efjsonOption_MULTILINE_STRING) && ul_likely(efjson__isNextLine(u))) {
      parser->state = (u == 0x0D /* '\r' */ ? efjsonVal__STRING_MULTILINE_CR : efjsonVal__STRING);
      token.type = efjsonType_STRING_NEXT_LINE;
    } else if((parser->option & efjsonOption_JSON5_STRING_ESCAPE) && u == 0x78 /* 'x' */) {
      parser->state = efjsonVal__STRING_ESCAPE_HEX;
      parser->substate = 0;
      parser->escape = 0;
      token.type = efjsonType_STRING_ESCAPE_HEX_START;
    } else token.extra = efjsonError_BAD_ESCAPE_IN_STRING;
    break;
  case efjsonVal__STRING_UNICODE:
    if(ul_likely(efjson__isHexDigit(u))) {
      parser->escape = efjson_cast(efjsonUint16, efjson_cast(efjsonUint16, parser->escape << 4) | efjson__hexDigit(u));
      token.type = efjsonType_STRING_ESCAPE_UNICODE;
      token.index = parser->substate;
      if((token.done = (++parser->substate == 4))) {
  #if EFJSON_CONF_COMBINE_ESCAPED_SURROGATE
        if(ul_unlikely(parser->escape >= 0xD800u && parser->escape <= 0xDBFFu)) {
          token.done = 0;
          parser->state = efjsonVal__STRING_UNICODE_NEXT;
          parser->prevPair = parser->escape;
          parser->substate = 0;
          return token;
        }
  #endif
  #if EFJSON_CONF_CHECK_ESCAPE_UTF
        if(ul_unlikely(efjson__isUtf16Surrogate(parser->escape))) {
          --parser->substate;
          parser->escape >>= 4;
          token.type = efjsonType_ERROR;
          token.extra = efjsonError_INVALID_ESCAPED_UTF;
          token.done = 0;
          token.index = 0;
          return token;
        }
  #endif
        parser->state = efjsonVal__STRING;
        token.extra = efjson_cast(efjsonUint16, parser->escape);
      }
    } else token.extra = efjsonError_BAD_UNICODE_ESCAPE_IN_STRING;
    break;
  #if EFJSON_CONF_COMBINE_ESCAPED_SURROGATE
  case efjsonVal__STRING_UNICODE_NEXT:
    switch(parser->substate) {
    case 0:
      if(ul_likely(u == 0x5C /* '\\' */)) {
        parser->substate = 1;
        token.index = 4;
        token.type = efjsonType_STRING_ESCAPE_UNICODE;
      } else token.extra = efjsonError_BAD_UNICODE_ESCAPE_IN_STRING;
      break;
    case 1:
      if(ul_likely(u == 0x75 /* 'u' */)) {
        parser->substate = 2;
        token.index = 5;
        token.type = efjsonType_STRING_ESCAPE_UNICODE;
        parser->escape = 0;
      } else token.extra = efjsonError_BAD_UNICODE_ESCAPE_IN_STRING;
      break;
    default:
      if(ul_likely(efjson__isHexDigit(u))) {
        parser->escape =
          efjson_cast(efjsonUint16, efjson_cast(efjsonUint16, parser->escape << 4) | efjson__hexDigit(u));
        token.type = efjsonType_STRING_ESCAPE_UNICODE;
        token.index = parser->substate + 4;
        if((token.done = (++parser->substate == 6))) {
          if(ul_likely(parser->escape >= 0xDC00u && parser->escape <= 0xDFFFu)) {
            parser->state = efjsonVal__STRING;
            token.extra =
              (efjson_cast(efjsonUint32, parser->prevPair & 0x3FFu) << 10 | (parser->escape & 0x3FFu)) + 0x10000u;
          } else {
            --parser->substate;
            parser->escape >>= 4;
            token.done = 0;
            token.index = 0;
            token.extra = efjsonError_INCOMPLETE_SURROGATE_PAIR;
          }
        }
      } else token.extra = efjsonError_BAD_UNICODE_ESCAPE_IN_STRING;
    }
    break;
  #endif
  case efjsonVal__STRING_ESCAPE_HEX:
    if(ul_likely(efjson__isHexDigit(u))) {
      parser->escape = efjson_cast(efjsonUint16, efjson_cast(efjsonUint16, parser->escape << 4) | efjson__hexDigit(u));
      token.type = efjsonType_STRING_ESCAPE_HEX;
      token.index = parser->substate;
      if((token.done = (++parser->substate == 2))) {
        parser->state = efjsonVal__STRING;
        token.extra = efjson_cast(efjsonUint16, parser->escape);
      }
    } else token.extra = efjsonError_BAD_HEX_ESCAPE_IN_STRING;
    break;

  case efjsonVal__NUMBER:
    if(u == 0x30 /* '0' */) {
      if(ul_unlikely(parser->substate == efjsonNumberState__ZERO)) token.extra = efjsonError_LEADING_ZERO_FORBIDDEN;
      else {
        if(parser->substate == efjsonNumberState__ONLY_SIGN) parser->substate = efjsonNumberState__ZERO;
        token.type = efjsonType_NUMBER_INTEGER_DIGIT;
      }
    } else if(u >= 0x31 /* '1' */ && u <= 0x39 /* '9' */) {
      if(ul_unlikely(parser->substate == efjsonNumberState__ZERO)) token.extra = efjsonError_LEADING_ZERO_FORBIDDEN;
      else {
        if(parser->substate == efjsonNumberState__ONLY_SIGN) parser->substate = efjsonNumberState__NON_LEADING_ZERO;
        token.type = efjsonType_NUMBER_INTEGER_DIGIT;
      }
    } else if(u == 0x2E /* '.' */) {
      if(ul_unlikely(parser->substate == efjsonNumberState__ONLY_SIGN)
         && !(parser->option & efjsonOption_EMPTY_INTEGER)) {
        token.extra = efjsonError_EMPTY_INTEGER_PART;
      } else {
        parser->state = efjsonVal__NUMBER_FRACTION;
        parser->substate = 0;
        token.type = efjsonType_NUMBER_FRACTION_START;
      }
    } else if(ul_unlikely(parser->substate == efjsonNumberState__ONLY_SIGN)) {
      if((parser->option & efjsonOption_INFINITY) && u == 0x49 /* 'I' */) {
        parser->state = efjsonVal__NUMBER_INFINITY;
        parser->substate = 1;
        token.type = efjsonType_NUMBER_INFINITY;
        token.index = 0;
      } else if((parser->option & efjsonOption_NAN) && u == 0x4E /* 'N' */) {
        parser->state = efjsonVal__NUMBER_NAN;
        parser->substate = 1;
        token.type = efjsonType_NUMBER_NAN;
        token.index = 0;
      } else token.extra = efjsonError_EMPTY_INTEGER_PART;
    } else {
      if(parser->substate == efjsonNumberState__ZERO) {
        if((parser->option & efjsonOption_HEXADECIMAL_INTEGER) && (u == 0x78 /* 'x */ || u == 0x58 /* 'X' */)) {
          parser->state = efjsonVal__NUMBER_HEX;
          parser->substate = 0;
          token.type = efjsonType_NUMBER_HEX_START;
          break;
        } else if((parser->option & efjsonOption_OCTAL_INTEGER) && (u == 0x6F /* 'o' */ || u == 0x4F /* 'O' */)) {
          parser->state = efjsonVal__NUMBER_OCT;
          parser->substate = 0;
          token.type = efjsonType_NUMBER_OCT_START;
          break;
        } else if((parser->option & efjsonOption_BINARY_INTEGER) && (u == 0x62 /* 'b' */ || u == 0x42 /* 'B' */)) {
          parser->state = efjsonVal__NUMBER_BIN;
          parser->substate = 0;
          token.type = efjsonType_NUMBER_BIN_START;
          break;
        }
      }

      if(u == 0x65 /* 'e' */ || u == 0x45 /* 'E' */) {
        parser->state = efjsonVal__NUMBER_EXPONENT;
        parser->substate = efjsonNumberExponent__NOT_YET;
        token.type = efjsonType_NUMBER_EXPONENT_START;
      } else if(ul_likely(efjson__isNumberSeparator(u, parser->option & efjsonOption_JSON5_WHITESPACE)))
        efjsonStreamParser__handleNumberSeparator(parser, u, &token);
      else token.extra = efjsonError_UNEXPECTED_IN_NUMBER;
    }
    break;
  case efjsonVal__NUMBER_FRACTION:
    if(u >= 0x30 /* '0' */ && u <= 0x39 /* '9' */) {
      parser->substate = 1;
      token.type = efjsonType_NUMBER_FRACTION_DIGIT;
    } else if(!parser->substate && !(parser->option & efjsonOption_EMPTY_FRACTION)) {
      token.extra = efjsonError_EMPTY_FRACTION_PART;
    } else if(u == 0x65 /* 'e' */ || u == 0x45 /* 'E' */) {
      parser->state = efjsonVal__NUMBER_EXPONENT;
      parser->substate = efjsonNumberExponent__NOT_YET;
      token.type = efjsonType_NUMBER_EXPONENT_START;
    } else if(ul_likely(efjson__isNumberSeparator(u, parser->option & efjsonOption_JSON5_WHITESPACE)))
      efjsonStreamParser__handleNumberSeparator(parser, u, &token);
    else token.extra = efjsonError_UNEXPECTED_IN_NUMBER;
    break;
  case efjsonVal__NUMBER_EXPONENT:
    if(u == 0x2B /* '+' */ || u == 0x2D /* '-' */) {
      if(parser->substate == efjsonNumberExponent__NOT_YET) {
        parser->substate = efjsonNumberExponent__AFTER_SIGN;
        token.type = efjsonType_NUMBER_EXPONENT_SIGN;
      } else token.extra = efjsonError_UNEXPECTED_IN_NUMBER;
    } else if(u >= 0x30 /* '0' */ && u <= 0x39 /* '9' */) {
      parser->substate = efjsonNumberExponent__AFTER_DIGIT;
      token.type = efjsonType_NUMBER_EXPONENT_DIGIT;
    } else if(parser->substate != efjsonNumberExponent__AFTER_DIGIT) token.extra = efjsonError_EMPTY_EXPONENT_PART;
    else if(ul_likely(efjson__isNumberSeparator(u, parser->option & efjsonOption_JSON5_WHITESPACE)))
      efjsonStreamParser__handleNumberSeparator(parser, u, &token);
    else token.extra = efjsonError_UNEXPECTED_IN_NUMBER;
    break;
  case efjsonVal__NUMBER_HEX:
    if(ul_likely(efjson__isHexDigit(u))) {
      parser->substate = 1;
      token.type = efjsonType_NUMBER_HEX;
    } else if(u == 0x2E /* '.' */) token.extra = efjsonError_FRACTION_NOT_ALLOWED;
    else if(parser->substate != 0) token.extra = efjsonError_EMPTY_INTEGER_PART;
    else if(ul_likely(efjson__isNumberSeparator(u, parser->option & efjsonOption_JSON5_WHITESPACE)))
      efjsonStreamParser__handleNumberSeparator(parser, u, &token);
    else token.extra = efjsonError_UNEXPECTED_IN_NUMBER;
    break;
  case efjsonVal__NUMBER_OCT:
    if(ul_likely(u >= 0x30 /* '0' */ && u <= 0x37 /* '7' */)) {
      parser->substate = 1;
      token.type = efjsonType_NUMBER_OCT;
    } else if(u == 0x65 /* 'e' */ || u == 0x45 /* 'E' */) token.extra = efjsonError_EXPONENT_NOT_ALLOWED;
    else if(u == 0x2E /* '.' */) token.extra = efjsonError_FRACTION_NOT_ALLOWED;
    else if(parser->substate != 0) token.extra = efjsonError_EMPTY_INTEGER_PART;
    else if(ul_likely(efjson__isNumberSeparator(u, parser->option & efjsonOption_JSON5_WHITESPACE)))
      efjsonStreamParser__handleNumberSeparator(parser, u, &token);
    else token.extra = efjsonError_UNEXPECTED_IN_NUMBER;
    break;
  case efjsonVal__NUMBER_BIN:
    if(ul_likely(u == 0x30 /* '0' */ || u == 0x31 /* '1' */)) {
      parser->substate = 1;
      token.type = efjsonType_NUMBER_BIN;
    } else if(u == 0x65 /* 'e' */ || u == 0x45 /* 'E' */) token.extra = efjsonError_EXPONENT_NOT_ALLOWED;
    else if(u == 0x2E /* '.' */) token.extra = efjsonError_FRACTION_NOT_ALLOWED;
    else if(parser->substate != 0) token.extra = efjsonError_EMPTY_INTEGER_PART;
    else if(ul_likely(efjson__isNumberSeparator(u, parser->option & efjsonOption_JSON5_WHITESPACE)))
      efjsonStreamParser__handleNumberSeparator(parser, u, &token);
    else token.extra = efjsonError_UNEXPECTED_IN_NUMBER;
    break;

  case efjsonVal__COMMENT_MAY_START:
    if((parser->option & efjsonOption_SINGLE_LINE_COMMENT) && u == 0x2F /* '/' */) {
      parser->state = efjsonVal__SINGLE_LINE_COMMENT;
      token.type = efjsonType_COMMENT_SINGLE_LINE;
    } else if(ul_likely((parser->option & efjsonOption_MULTI_LINE_COMMENT) && u == 0x2A /* '*' */)) {
      parser->state = efjsonVal__MULTI_LINE_COMMENT;
      token.type = efjsonType_COMMENT_MULTI_LINE;
    } else token.extra = efjsonError_COMMENT_FORBIDDEN;
    break;
  case efjsonVal__SINGLE_LINE_COMMENT:
    if(efjson__isNextLine(u)) parser->state = efjsonVal__EMPTY;
    token.type = efjsonType_COMMENT_SINGLE_LINE;
    break;
  case efjsonVal__MULTI_LINE_COMMENT:
    if(u == 0x2A /* '*' */) parser->state = efjsonVal__MULTI_LINE_COMMENT_MAY_END;
    token.type = efjsonType_COMMENT_MULTI_LINE;
    break;
  case efjsonVal__MULTI_LINE_COMMENT_MAY_END:
    if(u == 0x2F /* '/' */) {
      parser->state = efjsonVal__EMPTY;
      token.type = efjsonType_COMMENT_MULTI_LINE_END;
    } else {
      if(u != 0x2A /* '*' */) parser->state = efjsonVal__MULTI_LINE_COMMENT;
      token.type = efjsonType_COMMENT_MULTI_LINE;
    }
    break;

  case efjsonVal__IDENTIFIER:
    if(u == 0x3A /* ':' */) {
      parser->location = efjsonLoc__VALUE_START;
      parser->state = efjsonVal__EMPTY;
      token.location = efjsonLocation_OBJECT;
      token.type = efjsonType_OBJECT_VALUE_START;
    } else if(efjson_isWhitespace(u, (parser->option & efjsonOption_JSON5_WHITESPACE) != 0)) {
      parser->location = efjsonLoc__KEY_END;
      parser->state = efjsonVal__EMPTY;
      token.type = efjsonType_WHITESPACE;
    } else if(efjson_isIdentifierNext(u)) {
      token.type = efjsonType_IDENTIFIER_NORMAL;
    }
  #if EFJSON_CONF_CHECK_INPUT_UTF
    else if(ul_unlikely(efjson__isUtf16Surrogate(u) || u > 0x10FFFFu))
      token.extra = efjsonError_INVALID_INPUT_UTF;
  #endif
    else token.extra = efjsonError_INVALID_IDENTIFIER;
    break;
  case efjsonVal__IDENTIFIER_ESCAPE:
    if(parser->substate == 0) {
      if(ul_likely(u == 'u')) {
        parser->state = efjsonVal__IDENTIFIER_ESCAPE;
        parser->substate = 1;
        parser->escape = 0;
        token.type = efjsonType_IDENTIFIER_ESCAPE_START;
        token.index = 1;
        token.done = 1;
      } else token.extra = efjsonError_BAD_IDENTIFIER_ESCAPE;
    } else {
      if(ul_likely(efjson__isHexDigit(u))) {
        parser->escape =
          efjson_cast(efjsonUint16, efjson_cast(efjsonUint16, parser->escape << 4) | efjson__hexDigit(u));
        token.type = efjsonType_IDENTIFIER_ESCAPE;
        token.index = parser->substate - 1;
        if((token.done = (++parser->substate == 5))) {
  #if EFJSON_CONF_CHECK_ESCAPE_UTF
          if(ul_unlikely(efjson__isUtf16Surrogate(parser->escape))) {
            --parser->substate;
            parser->escape >>= 4;
            token.type = efjsonType_ERROR;
            token.extra = efjsonError_INVALID_ESCAPED_UTF;
            token.index = 0;
            token.done = 0;
            return token;
          }
  #endif
          parser->location = efjsonLoc__KEY_END;
          parser->state = efjsonVal__EMPTY;
          token.extra = efjson_cast(efjsonUint16, parser->escape);
        }
      } else token.extra = efjsonError_INVALID_IDENTIFIER_ESCAPE;
    }
  }
  return token;
}
  #if EFJSON_CONF_COMPRESS_STACK
    #undef efjson__bitshl
    #undef efjson___pushArray
    #undef efjson___pushObject
  #endif
  #undef efjson__stackLen
  #undef efjson__push
  #undef efjson__last
  #undef efjson__transformLocation
  #undef efjson__nextLocation
  #undef efjson__hexDigit
  #undef efjson__isUtf16Surrogate


EFJSON_PUBLIC size_t efjsonStreamParser_sizeof(void) {
  return sizeof(efjsonStreamParser);
}
EFJSON_PUBLIC void efjsonStreamParser_init(efjsonStreamParser* parser, efjsonUint32 option) {
  parser->position = parser->line = parser->column = 0;
  parser->option = option;
  parser->location = efjsonLoc__ROOT_START;
  parser->state = efjsonVal__EMPTY;
  parser->flag = 0;
  parser->len = 0;
  #if !(EFJSON_CONF_FIXED_STACK > 0)
  parser->cap = 0;
  parser->stack = NULL;
  #endif
}
EFJSON_PUBLIC void(efjsonStreamParser_deinit)(efjsonStreamParser* parser) {
  #if EFJSON_CONF_FIXED_STACK > 0
  (void)parser;
  #else
  free(parser->stack);
  parser->stack = NULL;
  parser->len = 0;
  parser->cap = 0;
  #endif
}
EFJSON_PUBLIC efjsonStreamParser* efjsonStreamParser_new(efjsonUint32 option) {
  efjsonStreamParser* parser = efjson_reptr(efjsonStreamParser*, malloc(sizeof(efjsonStreamParser)));
  if(ul_likely(parser != NULL)) efjsonStreamParser_init(parser, option);
  return parser;
}
EFJSON_PUBLIC void efjsonStreamParser_destroy(efjsonStreamParser* parser) {
  #if !(EFJSON_CONF_FIXED_STACK > 0)
  if(ul_likely(parser != NULL)) efjsonStreamParser_deinit(parser);
  #endif
  free(parser);
}
EFJSON_PUBLIC int(efjsonStreamParser_initCopy)(efjsonStreamParser* parser, const efjsonStreamParser* src) {
  if(parser != src) {
  #if EFJSON_CONF_FIXED_STACK > 0
    memcpy(parser, src, sizeof(efjsonStreamParser));
  #else
    efjsonUint8* stack = (efjsonUint8*)malloc(efjson_cast(efjsonStackLength, parser->cap));
    if(ul_unlikely(!stack)) return -1;
    memcpy(parser, src, sizeof(efjsonStreamParser));
    parser->stack = stack;
  #endif
  }
  return 0;
}
EFJSON_PUBLIC void(efjsonStreamParser_initMove)(efjsonStreamParser* parser, efjsonStreamParser* src) {
  if(parser != src) {
    memcpy(parser, src, sizeof(efjsonStreamParser));
  #if !(EFJSON_CONF_FIXED_STACK > 0)
    src->stack = NULL;
    parser->len = 0;
    src->cap = 0;
  #endif
  }
}
EFJSON_PUBLIC efjsonStreamParser* efjsonStreamParser_newCopy(const efjsonStreamParser* src) {
  efjsonStreamParser* parser = efjson_reptr(efjsonStreamParser*, malloc(sizeof(efjsonStreamParser)));
  if(ul_likely(parser != NULL)) {
  #if EFJSON_CONF_FIXED_STACK > 0
    memcpy(parser, src, sizeof(efjsonStreamParser));
  #else
    if(ul_unlikely(efjsonStreamParser_initCopy(parser, src) < 0)) {
      free(parser);
      return NULL;
    }
  #endif
  }
  return parser;
}


  #if EFJSON_CONF_CHECK_POSITION_OVERFLOW
    #define efjsonStreamParser__checkPosition(parser, uc, token, fail_stat)                 \
      if(ul_unlikely(((parser)->position == efjson_umax(efjsonPosition)) && ((uc) != 0))) { \
        memset(&(token), 0, sizeof(efjsonToken));                                           \
        (token).type = efjsonType_ERROR;                                                    \
        (token).extra = efjsonError_POSITION_OVERFLOW;                                      \
        fail_stat                                                                           \
      }                                                                                     \
      ((void)0)
  #else
    #define efjsonStreamParser__checkPosition(parser, uc, token, fail_stat)
  #endif
  #define efjsonStreamParser__movePosition(parser, uc)     \
    if(ul_unlikely((parser)->flag & efjsonFlag__MeetCr)) { \
      if(ul_unlikely((uc) != 0x0A /* '\n' */)) {           \
        ++(parser)->line;                                  \
        (parser)->column = 0;                              \
      }                                                    \
      (parser)->flag &= ~efjsonFlag__MeetCr;               \
    }                                                      \
    if(ul_likely((uc) != 0)) {                             \
      ++(parser)->position;                                \
      if(efjson__isNextLine((uc))) {                       \
        if((uc) == 0x0D /* '\r' */) {                      \
          ++(parser)->column;                              \
          (parser)->flag |= efjsonFlag__MeetCr;            \
        } else {                                           \
          ++(parser)->line;                                \
          (parser)->column = 0;                            \
        }                                                  \
      } else ++(parser)->column;                           \
    }                                                      \
    ((void)0)
EFJSON_PUBLIC efjsonToken efjsonStreamParser_feedOne(efjsonStreamParser* parser, efjsonUint32 u) {
  efjsonToken token;
  efjsonStreamParser__checkPosition(parser, u, token, return token;);
  token = efjsonStreamParser__step(parser, u);
  if(ul_likely(token.type != 0)) {
    efjsonStreamParser__movePosition(parser, u);
  }
  return token;
}
EFJSON_PUBLIC size_t
efjsonStreamParser_feed(efjsonStreamParser* parser, efjsonToken* dest, const efjsonUint32* src, size_t len) {
  size_t i;
  for(i = 0; i < len; ++i) {
    efjsonStreamParser__checkPosition(parser, src[i], dest[0], return 0;);
    dest[i] = efjsonStreamParser__step(parser, src[i]);
    if(ul_likely(dest[i].type != 0)) {
      efjsonStreamParser__movePosition(parser, src[i]);
    } else {
      dest[0] = dest[i];
      return 0;
    }
  }
  return i;
}
  #undef efjsonStreamParser__checkPosition
  #undef efjsonStreamParser__movePosition


EFJSON_PUBLIC efjsonPosition efjsonStreamParser_getLine(const efjsonStreamParser* parser) {
  return parser->line;
}
EFJSON_PUBLIC efjsonPosition efjsonStreamParser_getColumn(const efjsonStreamParser* parser) {
  return parser->column;
}
EFJSON_PUBLIC efjsonPosition efjsonStreamParser_getPosition(const efjsonStreamParser* parser) {
  return parser->position;
}
EFJSON_PUBLIC enum efjsonStage efjsonStreamParser_getStage(const efjsonStreamParser* parser) {
  if(parser->state == efjsonVal__EMPTY) return efjsonStage_PARSING;
  else if(parser->location == efjsonLoc__ROOT_START) return efjsonStage_NOT_STARTED;
  else if(parser->location == efjsonLoc__ROOT_END || parser->location == efjsonLoc__EOF) return efjsonStage_ENDED;
  else return efjsonStage_PARSING;
}


EFJSON_PUBLIC efjsonUint8 efjson_getError(efjsonToken token) {
  return token.type == efjsonType_ERROR ? token.extra : 0;
}


EFJSON_CODE_END
  #undef efjson_cast
  #undef efjson_reptr
  #undef efjson_umax
  #undef efjson_assert
  #undef efjson_condexpr
#endif /* EFJSON_STREAM_IMPL */


#endif /* EFJSON_STREAM_H */
