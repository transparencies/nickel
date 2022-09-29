//! Define the type of an identifier.
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

use crate::position::TermPos;

static INTERNER: Lazy<interner::Interner> = Lazy::new(interner::Interner::new);

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(into = "String", from = "String")]
pub struct Ident {
    label: interner::Symbol,
    pos: TermPos,
    generated: bool,
}

impl Ident {
    pub fn new_with_pos(label: impl AsRef<str>, pos: TermPos) -> Self {
        let generated = label.as_ref().starts_with(GEN_PREFIX);
        Self {
            label: INTERNER.intern(label),
            pos,
            generated,
        }
    }

    pub fn new(label: impl AsRef<str>) -> Self {
        Self::new_with_pos(label, TermPos::None)
    }

    pub fn label(&self) -> &str {
        INTERNER.lookup(self.label)
    }

    pub fn pos(&self) -> TermPos {
        self.pos
    }
}

/// Special character used for generating fresh identifiers. It must be syntactically impossible to
/// use to write in a standard Nickel program, to avoid name clashes.
pub const GEN_PREFIX: char = '%';

impl PartialOrd for Ident {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.label().partial_cmp(other.label())
    }
}

impl Ord for Ident {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.label().cmp(other.label())
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Eq for Ident {}

impl Hash for Ident {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl<F> From<F> for Ident
where
    String: From<F>,
{
    fn from(val: F) -> Self {
        Self::new(String::from(val))
    }
}

impl Into<String> for Ident {
    fn into(self) -> String {
        self.as_ref().to_owned()
    }
}

impl Ident {
    pub fn is_generated(&self) -> bool {
        self.generated
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.label()
    }
}

mod interner {
    use std::collections::HashMap;
    use std::sync::{Mutex, RwLock};

    use typed_arena::Arena;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Symbol(u32);

    pub(crate) struct Interner<'a>(RwLock<InnerInterner<'a>>);

    impl<'a> Interner<'a> {
        pub(crate) fn new() -> Self {
            Self(RwLock::new(InnerInterner::new()))
        }

        pub(crate) fn intern(&self, string: impl AsRef<str>) -> Symbol {
            self.0.write().unwrap().intern(string)
        }

        pub(crate) fn lookup(&self, sym: Symbol) -> &str {
            // SAFETY: We are making the returned &str lifetime the same as our struct,
            // which is okay here since the InnerInterner uses a typed_arena which prevents
            // deallocations, so the reference will be valid while the InnerInterner exists,
            // hence while the struct exists.
            unsafe { std::mem::transmute(self.0.read().unwrap().lookup(sym)) }
        }
    }

    struct InnerInterner<'a> {
        arena: Mutex<Arena<u8>>,
        map: HashMap<&'a str, Symbol>,
        vec: Vec<&'a str>,
    }

    impl<'a> InnerInterner<'a> {
        fn new() -> Self {
            Self {
                arena: Mutex::new(Arena::new()),
                map: HashMap::new(),
                vec: Vec::new(),
            }
        }

        fn intern(&mut self, string: impl AsRef<str>) -> Symbol {
            if let Some(sym) = self.map.get(string.as_ref()) {
                return *sym;
            }
            // SAFETY: here we are transmuting the reference lifetime:
            // &'arena str -> &'self str
            // This is okay since the lifetime of the arena is identical to the one of the struct.
            // It is also okay to use it from inside the mutex, since typed_arena does not allow deallocation,
            // so references are valid until the arena drop, which is tied to the struct drop.
            let in_string = unsafe {
                std::mem::transmute(self.arena.lock().unwrap().alloc_str(string.as_ref()))
            };
            let sym = Symbol(self.vec.len() as u32);
            self.vec.push(in_string);
            self.map.insert(in_string, sym);
            sym
        }

        fn lookup(&self, sym: Symbol) -> &str {
            self.vec[sym.0 as usize]
        }
    }
}
