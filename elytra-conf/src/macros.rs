#[macro_export]
macro_rules! indexed_entry {
    ($indexty:ty: $name:ident { $( $s:ident: $sx:expr ),+ }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            $(
                $s = ${index()},
            )*
        }
        impl $indexty for $name {
            fn as_index(self) -> usize {
                self as usize
            }
            fn from_byte(byte: u8) -> Option<Self> {
                match byte {
                    $(
                        ${index()} => Some(Self::$s),
                    )*
                    _ => None
                }
            }
            fn get_entry(self) -> &'static $crate::entry::EntryDesc {
                &Self::ENTRIES[self.as_index()]
            }
            fn count() -> usize {
                Self::ENTRIES.len()
            }
        }
        impl $name {
            pub const ENTRIES: [$crate::entry::EntryDesc; ${count($sx)}] = [$(
                $sx.as_entry(),
            )*];
        }
    };
    ($indexty:ty: $name:ident) => {
        #[derive(Debug, PartialEq, Eq, Copy, Clone)]
        pub struct $name;
        impl $indexty for $name {
            fn as_index(self) -> usize {
                panic!("empty index")
            }
            fn from_byte(_: u8) -> Option<Self> {
                None
            }
            fn get_entry(self) -> &'static $crate::entry::EntryDesc {
                panic!("empty index")
            }
            fn count() -> usize {
                0
            }
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! sections {
    ($name:ident {}) => {
        indexed_entry!($crate::SectionIndex: $name);
    };
    ($name:ident) => {
        indexed_entry!($crate::SectionIndex: $name);
    };
    ($name:ident { $($s:tt)+ }) => {
        indexed_entry!($crate::SectionIndex: $name { $($s)+ } );
    };
}

#[macro_export(local_inner_macros)]
macro_rules! actions {
    ($name:ident {}) => {
        indexed_entry!($crate::ActionIndex: $name);
    };
    ($name:ident) => {
        indexed_entry!($crate::ActionIndex: $name);
    };
    ($name:ident { $($s:tt)+ }) => {
        indexed_entry!{$crate::ActionIndex: $name { $($s)+ }}
    };
}

#[macro_export(local_inner_macros)]
macro_rules! infos {
    ($name:ident {}) => {
        indexed_entry!($crate::InfoIndex: $name)
    };
    ($name:ident) => {
        indexed_entry!($crate::InfoIndex: $name)
    };
    ($name:ident { $($s:tt)+ }) => {
        indexed_entry!($crate::InfoIndex: $name { $($s)+ } );
    };
}

#[macro_export(local_inner_macros)]
macro_rules! props {
    ($name:ident {}) => {
        indexed_entry!($crate::PropIndex: $name);
    };
    ($name:ident) => {
        indexed_entry!($crate::PropIndex: $name);
    };
    ($name:ident { $($s:tt)+ }) => {
        indexed_entry!($crate::PropIndex: $name { $($s)+ } );
    };
}

#[macro_export(local_inner_macros)]
macro_rules! elytra {
    ($cvis:vis $cident:ident: $tident:ident {
        info: $info:ty,
        props: $props:ty,
        sections: $sections:ty,
        actions: $actions:ty,
        layout: { $( $ls:path: [ $( $lf:expr ),* ] ),* }
        }
    ) => {
        pub type $tident = $crate::config::Config<${count($lf)}, $sections, $props, $info, $actions>;
        $cvis const $cident: $tident = $crate::config::Config::new(
            [$(
                $(
                ($ls, $lf),
                )*
            )*],
        );
    };
    ( $cvis:vis $cident:ident: $tident:ident {
        info: { $($ix:tt)+ },
        props: { $($px:tt)+ },
        sections: { $($sx:tt)+ },
        actions: { $($ax:tt)+ },
        layout: { $($lx:tt)+ }
     }) => {
        elytra!($vis $cident: $tident {
            info: InfoField { $($ix)+ },
            props: PropField { $($px)+ },
            sections: Section { $($sx)+ },
            actions: Action { $($ax)+ },
            layout: { $($lx)+ }
        });
    };
    ( $cvis:vis $cident:ident: $tident:ident {
        info: $i:ident { $($ix:tt)* },
        props: $p:ident { $($px:tt)* },
        sections: $s:ident { $($sx:tt)* },
        actions: $a:ident { $($ax:tt)* },
        layout: { $( $ls:path: [ $( $lf:expr ),* ] ),* }
    }
    ) => {
        actions!($a { $($ax)* });
        sections!($s { $($sx)* });
        infos!($i { $($ix)* });
        props!($p { $($px)* });

        pub type $tident = $crate::config::Config<${count($lf)}, $s, $p, $i, $a>;
        $cvis const $cident: $tident = $crate::config::Config::new(
            [$(
                $(
                ($ls, $lf),
                )*
            )*],
        );
    };
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_empty() {

        elytra!(C: T {
            info: InfoField { },
            props: PropField { },
            sections: Section {  },
            actions: Action { },
            layout: {}
        });

        assert_eq!(0, InfoField::count());
        assert_eq!(0, PropField::count());
        assert_eq!(0, Section::count());
        assert_eq!(0, Action::count());
        assert_eq!(0, C.layout.len());
    }

    #[test]
    fn test_some_entries() {
        elytra!(C: T {
            info: InfoField {
                Foo: info("Foo")
            },
            props: PropField {
                One: prop("One"),
                Two: prop("Two")
            },
            sections: Section {
                Top: section("Top"),
                Mid: section("Mid"),
                Bot: section("Bot")
            },
            actions: Action {
                Begin: action("Being"),
                End: action("End")
            },
            layout: {
                Section::Top: [
                    Field::Info(InfoField::Foo)
                ],
                Section::Mid: [
                    Field::Prop(PropField::One)
                ],
                Section::Bot: [
                    Field::Prop(PropField::Two)
                ]
            }
        });

        assert_eq!(1, InfoField::count());
        assert_eq!(2, PropField::count());
        assert_eq!(3, Section::count());
        assert_eq!(2, Action::count());
        assert_eq!([
            (Section::Top, Field::Info(InfoField::Foo)),
            (Section::Mid, Field::Prop(PropField::One)),
            (Section::Bot, Field::Prop(PropField::Two)),
        ], C.layout);
    }

    #[test]
    fn test_sections_macro() {
        sections!( S {
            Top: section("Top"),
            Mid: section("Mid"),
            Bot: section("Bot")
        });
        assert_eq!(3, S::count());
        assert_eq!("Top", S::ENTRIES[0].name);
    }
}