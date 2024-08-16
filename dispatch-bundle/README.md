# dispatch-bundle

A multi-type container with a static size.

## no_std

This crate is intended for use in `no_std` environments.

# Motivation

Trait objects require dynamic memory allocation since the concrete type being dispatched is not known until runtime.

For `no_std` environments, a global allocator may be limited, or not available at all. But dynamic dispatch may still be desired.

# Solution

This crate provides an attribute macro for generating an enum type that contains a finite set of concrete types which implement a common trait.

Each variant of the enum corresponds to a type, in the form of a single item tuple variant.

The memory footprint of an enum is proportional to the largest variant:

```rust
enum MultipleTypes {
  A(A),
  B(B),
  C(C),
  D(D)
}
```

Memory:

```
  |#     <- A
  |###   <- B
  |##    <- C
  |##### <- D
##|##### < Total size
 ^
 |
padding/tag
```

# Usage

## Basic

To create a bundle, simply mark an enum as a such:

```rust
trait Foo {
    fn bar(&self) -> u8;
}

#[bundle(Foo)]
enum MyBundle {
  FirstType,
  SecondType,
  ThirdType
}
```

Now you can invoke methods defined by the shared trait:

```rust
impl Foo for FirstType {
    fn bar(&self) -> u8 {
        0
    }
}

impl Foo for SecondType {
    fn bar(&self) -> u8 {
        1
    }
}

impl Foo for ThirdType {
    fn bar(&self) -> u8 {
        2
    }
}

let bundle: MyBundle = { /* fetch bundle from somewhere... */ }

let bar = bundle.inner().bar(); // will be 0, 1, or 2 depending on what's in the bundle
```

## Other macros

Bundles can still be used with other macros as long as the bundle is the first one executed.

For example, use with `derive` where types `A`, `B` and `C` implement `Clone`:

```rust
#[bundle(SomeTrait)] // this goes first so derive sees the transformed enum
#[derive(Clone)]
enum MyBundle {
    A,
    B,
    C,
}
```

## Generics

If generics are required for your bundle, you can add them like so:

```rust
#[bundle(BundleTrait)]
enum MyBundle<T: Trait1, U: Trait2> {
    A(A<T>), // notice you must now create the tuple variant yourself
    B,
    C(C<U>),
}
```

# Design Considerations

## Performance

Using a bundle requires 2 look ups:

1. Matching the enum
2. Consulting the vtable for dispatch

The optimizer may realize these two lookups are always the same and optimize it out.

## Safety

The `#[bundle(...)]` macro cannot generate unsafe code.
