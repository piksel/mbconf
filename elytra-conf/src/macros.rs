
#[macro_export]
macro_rules! sections {
    ($( $s:ident: $sx:expr ),* ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Section {
            $(
                $s = ${index()},
            )*
        }
        impl $crate::SectionIndex for Section {
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
        }
        impl Section {
            pub const ENTRIES: [$crate::entry::EntryDesc; ${count($sx)}] = [$(
                $sx.as_entry(),
            )*];
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! elytra {
    ( $cvis:vis $cident:ident: $tident:ident {
        info: { $( $i:ident: $ix:expr ),* },
        props: { $( $f:ident: $fx:expr ),* },
        sections: { $( $s:ident: $sx:expr ),* },
        actions: { $( $a:ident: $an:expr ),* },
        layout: { $( $ls:path: [ $( $lf:expr ),* ] ),* }
        }
    ) => {

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Action {
            $(
                $a = ${index()},
            )*
        }
        impl $crate::traits::ActionIndex for Action {
            fn as_index(self) -> usize {
                self as usize
            }
            fn from_byte(byte: u8) -> Option<Self> {
                match byte {
                    $(
                        ${index()} => Some(Self::$a),
                    )*
                    _ => None
                }
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum PropField {
            $(
                $f = ${index()},
            )*
        }

        impl $crate::traits::PropIndex for PropField {
            fn as_index(self) -> usize {
                self as usize
            }
            fn from_byte(byte: u8) -> Option<Self> {
                match byte {
                    $(
                        ${index()} => Some(Self::$f),
                    )*
                    _ => None
                }
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Section {
            $(
                $s = ${index()},
            )*
        }
        impl $crate::traits::SectionIndex for Section {
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
        }

        // #[repr(u8)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum InfoField {
            $(
                $i = ${index()},
            )*
        }
        impl $crate::traits::InfoIndex for InfoField {
            fn as_index(self) -> usize {
                self as usize
            }
            fn from_byte(byte: u8) -> Option<Self> {
                match byte {
                    $(
                        ${index()} => Some(Self::$i),
                    )*
                    _ => None
                }
            }
        }

        pub type $tident = $crate::config::Config<${count($s)}, ${count($f)}, ${count($i)}, ${count($a)}, ${count($lf)}, Section, PropField, InfoField, Action>;
        $cvis const $cident: $tident = $crate::config::Config::new(            
            [$(
                $sx.as_entry(),
            )*],
            [$(
                $fx.as_entry(),
            )*],
            [$(
                $ix.as_entry(),
            )*],
            [$(
                $an.as_entry(),
            )*],
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
            info: {},
            props: {},
            sections: {},
            actions: {},
            layout: {}
        });

        assert_eq!(0, C.info_fields.len());
        assert_eq!(0, C.prop_fields.len());
        assert_eq!(0, C.sections.len());
        assert_eq!(0, C.actions.len());
        assert_eq!(0, C.layout.len());
    }

    #[test]
    fn test_some_entries() {
        elytra!(C: T {
            info: {
                Foo: info("Foo")
            },
            props: {
                One: prop("One"),
                Two: prop("Two")
            },
            sections: {
                Top: section("Top"),
                Mid: section("Mid"),
                Bot: section("Bot")
            },
            actions: {
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

        assert_eq!(1, C.info_fields.len());
        assert_eq!(2, C.prop_fields.len());
        assert_eq!(3, C.sections.len());
        assert_eq!(2, C.actions.len());
        assert_eq!([
            (Section::Top, Field::Info(InfoField::Foo)),
            (Section::Mid, Field::Prop(PropField::One)),
            (Section::Bot, Field::Prop(PropField::Two)),
        ], C.layout);
    }

    #[test]
    fn test_sections_macro() {
        sections!(
            Top: section("Top"),
            Mid: section("Mid"),
            Bot: section("Bot")
        );
        assert_eq!(3, Section::ENTRIES.len());
        assert_eq!("Top", Section::ENTRIES[0].name);
    }
}