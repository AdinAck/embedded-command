#![no_std]

pub use macros::bundle;

#[cfg(test)]
mod tests {
    use cookie_cutter::{encoding::vanilla, SerializeBuf};
    use macros::bundle;

    trait Foo {
        fn bar(&self) -> u8;
    }

    #[derive(Clone)]
    struct A;
    #[derive(Clone)]
    struct B;
    #[derive(Clone)]
    struct C;

    impl Foo for A {
        fn bar(&self) -> u8 {
            0
        }
    }

    impl Foo for B {
        fn bar(&self) -> u8 {
            1
        }
    }

    impl Foo for C {
        fn bar(&self) -> u8 {
            2
        }
    }

    #[test]
    fn basic() {
        #[bundle(Foo)]
        enum MyBundle {
            A,
            B,
            C,
        }

        let mut bundle = MyBundle::B(B);

        assert_eq!(1, bundle.inner().bar());
    }

    #[test]
    fn conversion() {
        #[bundle(Foo)]
        enum MyBundle {
            A,
            B,
            C,
        }

        let mut bundle: MyBundle = A.into();

        assert_eq!(0, bundle.inner().bar());
    }

    #[test]
    fn derive() {
        #[bundle(Foo)]
        #[derive(Clone)]
        enum MyBundle {
            A,
            B,
            C,
        }

        let bundle = MyBundle::C(C);

        assert_eq!(2, bundle.clone().inner().bar());
    }

    #[test]
    fn cookie_cutter() {
        #[derive(vanilla::SerializeIter, vanilla::SerializeBuf)]
        struct A {
            val: u8,
        }

        #[derive(vanilla::SerializeIter, vanilla::SerializeBuf)]
        struct B {
            val: u16,
        }

        #[derive(vanilla::SerializeIter, vanilla::SerializeBuf)]
        struct C {
            val: u8,
            other: A,
        }

        impl Foo for A {
            fn bar(&self) -> u8 {
                self.val
            }
        }

        impl Foo for B {
            fn bar(&self) -> u8 {
                self.val as u8
            }
        }

        impl Foo for C {
            fn bar(&self) -> u8 {
                self.val + self.other.val
            }
        }

        const TEN: u8 = 10;

        #[bundle(Foo)]
        #[derive(vanilla::SerializeIter, vanilla::SerializeBuf)]
        #[repr(u8)]
        enum MyBundle {
            A,
            B,
            C = TEN,
        }

        let mut buf = <MyBundle as SerializeBuf>::Serialized::default();

        MyBundle::C(C {
            val: 15,
            other: A { val: 20 },
        })
        .serialize_buf(&mut buf);

        assert_eq!([10, 15, 20], buf);
    }

    #[test]
    fn generics() {
        trait Foo {}
        trait Bar {}

        impl Foo for u8 {}
        impl Bar for u8 {}

        struct A<T: Bar> {
            #[allow(unused)]
            val: T,
        }
        struct B;
        struct C<T: Foo> {
            #[allow(unused)]
            val: T,
        }

        impl<T: Bar> Foo for A<T> {}
        impl Foo for B {}
        impl<T: Foo> Foo for C<T> {}

        #[bundle(Foo)]
        enum MyBundle<T: Bar, U: Foo> {
            A(A<T>),
            B,
            C(C<U>),
        }

        let _bundle: MyBundle<_, B> = A { val: 0u8 }.into();
        let _bundle: MyBundle<u8, _> = C { val: B }.into();
    }
}
