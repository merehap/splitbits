# Concise bit field extraction
[![Doc Status]][docs] [![Latest Version]][crates.io]

[doc status]: https://img.shields.io/docsrs/splitbits?style=plastic
[docs]: https://docs.rs/splitbits/
[latest version]: https://img.shields.io/crates/v/splitbits?style=plastic
[crates.io]: https://crates.io/crates/splitbits

<img width="100%" alt="splitbits"
  src="https://github.com/merehap/splitbits/blob/master/static/splitbits.jpg">

Splitbits provides concise macros for extracting bits from integers and combining bits into
integers.
```rust
use splitbits::splitbits;

// Parse the template provided ("aaabbbbb"), apply it to the input, then generate a struct
// populated with the bit field values.
let fields = splitbits!(0b11110000, "aaabbbbb");
// Single-letter field names, generated from the unique letters in the template above.
assert_eq!(fields.a, 0b111);
assert_eq!(fields.b, 0b10000);
```

# Why use splitbits?
Splitbits allows you to skip tedious, error-prone bit operations, instead providing a
simple, terse, and readable template format for specifying which bits correspond to which
fields.

Splitbits is intended for cases where [bitfield] is too heavy-weight: when you don't want to
explicitly declare a new struct for data that you won't use as a return value or argument.
Splitbits also provides some features that are arguably out of scope for [bitfield].

[bitfield]: https://docs.rs/bitfield/latest/bitfield/

# The four base macros
For additional examples, see each macro's page.
- [`splitbits!`] - Extract bit fields out of an integer, storing them as fields of a struct.
(See example above.)
- [`combinebits!`] - Combine bits of multiple integers into a single integer.
  ```rust
  use splitbits::combinebits;

  let b: u8 = 0b1010_1010;
  let m: u8 = 0b1111;
  let e: u8 = 0b0000;
  let result = combinebits!("bbbb bbbb mmmm eeee");
  assert_eq!(result,       0b1010_1010_1111_0000);
  ```
- [`splitbits_then_combine!`] - Extract bit fields from multiple integers then combine them
into a single integer.
  ```rust
  use splitbits::splitbits_then_combine;

  let output = splitbits_then_combine!(
      0b1111_0000, "aaaa ..bb", // input 0, input template 0,
      0b1011_1111, "cc.. ....", // input 1, input template 1
                   "aaaa bbcc", // output template
  );
  assert_eq!(output, 0b1111_0010);
  ```
- [`replacebits!`] - Replace some of the bits of an integer with bits from other integers.
  ```rust
  use splitbits::replacebits;

  let a: u16 = 0b101;
  let b: u8 = 0b01;
  // Placeholder periods in the template are the bits that will not be replaced.
  let result = replacebits!(0b10000001, "aaa..bb.");
  assert_eq!(result,                   0b10100011);
  ```

# Macro variants
The four base macros cover all the basic functionality that this crate offers and should be
sufficient for most use-cases. However, in many situations better ergonomics can be achieved by
using the macro variants.
#### Hexadecimal
All four base macros have equivalents that use hexadecimal digits for their templates rather
than bits (binary digits). The variants are [`splithex!`], [`combinehex!`],
[`splithex_then_combine!`], and [`replacehex!`].

#### Splitbits variants
[`splitbits!`] itself has many variants which are intended for better ergonomics for the generated
variables. The basic variants are:
- [`splitbits_named!`] - Used when single-letter variable names aren't descriptive enough. This
variant returns a tuple (instead of a struct) of the resulting fields, allowing the caller to
assign individual long field names in the `let` binding.
- [`splitbits_named_into!`] - Same as [`splitbits_named!`] except that the caller specifies the
types of the resulting fields, not just their names. `into()` is called on each tuple field
before it reaches the caller. This is useful for when the default type (the smallest integer
type that will fit the field) is a smaller type than the caller would like to use, or if the
caller has a newtype that they would like to use instead.
- [`splitbits_ux!`] - Used when exact-width integers (e.g. u4, u7, u20) are needed, instead of
just the standard types (u8, u16, u32, u64, u128, and bool). Requires the [ux] crate.

[ux]: <https://docs.rs/ux/latest/ux/>
# Template syntax
Templates are a string of characters that represent the names and bit-placements of fields
within an integer.

Example: `"..aa bccc dddd 1010"`

The possible elements of a template are:
- Names - a single letter that indicates the name of a field. (Currently only ASCII allowed.)
- Placeholders - a period that indicates a digit that will be ignored.
- Literals - a literal digit of the numeric base of the template (e.g. binary or hexadecimal).
- Whitespaces - an empty space character used to make formatting more human-friendly,
paralleling how underscores can be added to integer literals.

The bits of a field are usually contiguous within a template, but they don't have to be:
`"aabbbbaa"`. This template will interpret `a` as a single field, with no bits present between
the halves.

#### Restrictions
- Templates (currently) must have a standard integer width (8, 16, 32, 64, or 128 bits).
- Placeholders cannot be used in the template for [`combinebits!`], nor in the output template
of [`splitbits_then_combine!`]. They are not meaningful in those contexts.
- Literals (currently) cannot be used in the template for [`splitbits!`] nor the input templates
of [`splitbits_then_combine!`]. In the future, literals could be used in these contexts for
input validation.

# Settings
There are currently two settings that can be passed to change the behavior of the various
macros:
- **min** - sets the minimum size of variable that can be produced by the [`splitbits!`] family of
macros. Must be set if you don't want booleans generated for 1-bit fields.
  - For standard (non-ux) macros, the valid setting values are `bool` (the default), `u8`, `u16`, `u32`,
`u64`, and `u128`. See examples at [`splitbits!`].
  - For ux macros, the valid setting values are `bool` (the default) or `uX`, where X is
  between 1 and 128 (both inclusive). See examples at [`splitbits_ux!`].
- **overflow** - sets the behavior to use if the value of an input variable is larger than the
corresponding slot in the template. Used in [`combinebits!`] and [`replacebits!`]. Valid
setting values are `truncate` (the default), `panic`, `corrupt`, or `saturate`.

# Milestones for future versions
### User-facing
- Support template bases beyond binary and hexadecimal (base 8, 32, and 64).
- Add setting for validating splitbits inputs by specifying literals in the template.
- Add file-level config for setting defaults for the overflow and min settings.
  - Will allow macro invocations to be more concise at the call-site when the default settings are not desired for a project.
- Allow non-standard template lengths

### Code quality
- Represent output as a syntax tree before final code generation.
  - Will enable cleaner generated code.
  - Will enable better optimization for generated code.
  - Will make adding new features easier.
  - Will improve code clarity and decrease bugginess by disentangling separate concerns.
  - Will fix bug where combinebits input types must not be larger than the template type.
- Extract argument parsing from business logic.
  - Will improve code clarity and error handling.
