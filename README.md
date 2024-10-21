# Concise bit field extraction
[![Doc Status]][docs] [![Latest Version]][crates.io]

[doc status]: https://img.shields.io/docsrs/splitbits?style=plastic
[docs]: https://docs.rs/splitbits/
[latest version]: https://img.shields.io/crates/v/splitbits?style=plastic
[crates.io]: https://crates.io/crates/splitbits

<img width="100%" alt="splitbits"
  src="https://github.com/merehap/splitbits/blob/master/static/splitbits.jpg">

Concise macros for extracting bits from integers and combining bits into integers. No scary syntax.
Minimal magic.
```rust
use splitbits::splitbits;

// Parse the template ("aaabbbbb"), apply it to the input,
// then generate a struct populated with the bit field values.
let fields = splitbits!(0b11110000, "aaabbbbb");
// Single-letter field names,
// generated from the unique letters in the template above.
assert_eq!(fields.a, 0b111);
assert_eq!(fields.b, 0b10000);
```

# Why use splitbits?
Splitbits replaces tedious, error-prone bit operations with a simple template format, making it
easy to extract bits into variables.

Every operation than can be executed at compile time is. Generated code should be as efficient
as hand-written bit operations.

Splitbits is intended for cases where [bitfield] is too heavy-weight syntactically: when you
don't want to explicitly declare a new struct for data that you won't use as a return value or
argument.

[bitfield]: https://docs.rs/bitfield/latest/bitfield/

# The four base macros
For additional examples, see each macro's page.
- [splitbits!] - Extract bit fields out of an integer, storing them as fields of a struct.
(See example above.) By default, each field will be stored in the smallest unsigned integer type
possible.
- [combinebits!] - Combine bits of multiple integers into a single integer.
  ```rust
  use splitbits::combinebits;

  let b: u8 = 0b1010_1010;
  let m: u8 = 0b1111;
  let e: u8 = 0b0000;
  let result = combinebits!("bbbb bbbb mmmm eeee");
  assert_eq!(result,       0b1010_1010_1111_0000);
  ```
- [splitbits_then_combine!] - Extract bit fields from multiple integers then combine them
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
- [replacebits!] - Replace some of the bits of an integer with bits from other integers.
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
using these more specialized macro variants.
#### Hexadecimal
All four base macros have equivalents that use hexadecimal digits for their templates rather
than bits (binary digits). The variants are [splithex!], [combinehex!],
[splithex_then_combine!], and [replacehex!].

#### Splitbits variants
[splitbits!] itself has many variants which are intended for better ergonomics for the generated
variables. The basic variants are:
- [splitbits_named!] - Used when single-letter variable names aren't descriptive enough. This
variant returns a tuple (instead of a struct) of the resulting fields, allowing the caller to
assign individual long field names in the `let` binding.
- [splitbits_named_into!] - Same as [splitbits_named!] except that the caller specifies the
types of the resulting fields, not just their names. `into()` is called on each tuple field
before it reaches the caller. This is useful for when the default type (the smallest integer
type that will fit the field) is a smaller type than the caller would like to use, or if the
caller has a newtype that they would like to use instead.
- [splitbits_ux!] - Used when exact-width integers (e.g. u4, u7, u20) are needed, instead of
just the standard types (u8, u16, u32, u64, u128, and bool). Requires the [ux] crate.

[ux]: <https://docs.rs/ux/latest/ux/>

# Documentation
Find thorough documentation of this crate and its many macro variants [here], including detailed template syntax, settings, and per-macro documentation and examples.

[here]: <https://docs.rs/splitbits>

# Milestones for future versions
### User-facing
- Support template bases beyond binary and hexadecimal (base 8, 32, and 64).
- Add setting for validating splitbits inputs by specifying literals in the template.
- Add file-level config for setting defaults for the overflow and min settings.
  - Will allow macro invocations to be more concise at the call-site when the default settings are not desired for a project.
- Allow non-standard template lengths.
- Verify that splitbits can be used in no_std environments. It seems that the package itself doesn't need to be no_std, just the generated code.
- Add support for different endianness for inputs and outputs?

### Performance
- Add tests that verify that the intended code for each macro is what is generated.
  - Verify at the generated rust level.
  - Verify at the assembly level
    - Is the generated splitbits! struct elided or are we taking a performance hit?
    - Will be difficult due to different CPU architectures and instruction sets.
- Remove all chained shift operations where possible
- Always use overflow=corrupt for combinebits! and replacebits! if the input variable size exactly matches the field slot size.

### Code quality
- Represent output as a syntax tree before final code generation.
  - Will enable cleaner generated code.
  - Will enable better optimization for generated code.
  - Will make adding new features easier.
  - Will improve code clarity and decrease bugginess by disentangling separate concerns.
  - Will fix bug where combinebits input types must not be larger than the template type.
- Extract argument parsing from business logic.
  - Will improve code clarity and error handling.

[splitbits!]: <https://docs.rs/splitbits/latest/splitbits/macro.splitbits.html>
[combinebits!]: <https://docs.rs/splitbits/latest/splitbits/macro.combinebits.html>
[splitbits_then_combine!]: <https://docs.rs/splitbits/latest/splitbits/macro.splitbits_then_combine.html>
[replacebits!]: <https://docs.rs/splitbits/latest/splitbits/macro.replacebits.html>
[splitbits_named!]: <https://docs.rs/splitbits/latest/splitbits/macro.splitbits_named.html>
[splitbits_named_into!]: <https://docs.rs/splitbits/latest/splitbits/macro.splitbits_named_into.html>
[splitbits_ux!]: <https://docs.rs/splitbits/latest/splitbits/macro.splitbits_ux.html>
[splithex!]: <https://docs.rs/splitbits/latest/splitbits/macro.splithex.html>
[combinehex!]: <https://docs.rs/splitbits/latest/splitbits/macro.combinehex.html>
[splithex_then_combine!]: <https://docs.rs/splitbits/latest/splitbits/macro.splithex_then_combine.html>
[replacehex!]: <https://docs.rs/splitbits/latest/splitbits/macro.replacehex.html>
