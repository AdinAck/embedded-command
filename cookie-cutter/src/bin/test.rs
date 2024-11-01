#![no_std]
#![no_main]

use core::ptr::{null, null_mut, read_volatile, write_volatile};

use panic_halt as _;

use cookie_cutter::{encoding::vanilla, SerializeIter};
use cortex_m_rt::entry;

#[derive(vanilla::SerializeIter)]
struct Foo {
    a: u32,
    b: u16,
}

#[inline(never)]
fn deserialize<const N: usize>(buf: &[u8; N]) -> Foo {
    unsafe { Foo::deserialize_iter(buf).unwrap_unchecked() }
}

#[inline(never)]
fn serialize<const N: usize>(foo: Foo, buf: &mut [u8; N]) {
    unsafe { foo.serialize_iter(buf).unwrap_unchecked() }
}

#[entry]
fn main() -> ! {
    let buf = unsafe { read_volatile(null() as *const [u8; 20]) };

    let foo = deserialize(&buf);

    let mut buf = [0; 20];

    serialize(foo, &mut buf);

    unsafe { write_volatile(null_mut(), buf) };

    loop {}
}
