# Serac

A static, modular, and light-weight serialization framework.

*Encoders* specify what kind of serialization medium they can target, and define the
serialization or deserialization process for that medium.

Serac comes with one built-in encoder *Vanilla*.

## Why not serde?

Serac's value proposition is its ability to perform static analysis on serialization
participants to either refine the failure space, or guarantee infallibility.

## Examples

### Serialize a number

```rust
use serac::{buf, SerializeIter};

let number = 0xdeadbeefu32;

let mut buf = buf!(u32); // [0; 4] in this case.
number.serialize_iter(&mut buf).unwrap();

let readback = SerializeIter::deserialize_iter(&buf).unwrap();
assert_eq(number, readback);
```
> The "Vanilla" encoding is used by default unless otherwise specified.

In the above example, there was an `unwrap` used on the result of `serialize_iter`
because the buffer *could* have been too short.

Using `SerializeBuf`, we can avoid this:

```rust
use serac::{buf, SerializeBuf};

let number = 0xdeadbeefu32;

let mut buf = buf!(u32);
number.serialize_buf(&mut buf); // it is statically known that "u32" fits in "[u8; 4]"

// it is *not* statically known that all values of "[u8; 4]" produce a valid "u32",
// and only that failure mode is expressed
let readback = SerializeBuf::deserialize_buf(&buf).unwrap();
assert_eq(number, readback);
```

Many built in types like numbers, tuples, and arrays implement both `SerializeIter`
and `SerializeBuf`.

### Serialize a custom type

```rust
use serac::{buf, encoding::vanilla, SerializeBuf};

const BE: u8 = 0xbe;

#[repr(u8)]
#[derive(Debug, PartialEq, vanilla::SerializeIter, vanilla::SerializeBuf)]
enum Foo {
    A,
    B(u8, i16) = 0xde,
    C,
    D { bar: u16, t: i8 } = BE,
}

let foo = Foo::D { bar: 0xaa, t: -1 };

let mut buf = buf!(Foo);
foo.serialize_buf(&mut buf);

let readback = SerializeBuf::deserialize_buf(&buf).unwrap();
assert_eq(foo, readback);
```

This example shows a crazy enum with lots of fancy things going on, which is able
to be serialized by serac.
