//! Define the main evaluation stack of the Nickel abstract machine and related operations.
//!
//! See [eval](../eval/index.html).
use crate::eval::Closure;
use crate::operation::OperationCont;
use crate::position::RawSpan;
use std::cell::RefCell;
use std::rc::Weak;

/// An element of the stack.
#[derive(Debug)]
pub enum Marker {
    /// An argument of an application.
    Arg(Closure, Option<RawSpan>),
    /// A thunk, which is pointer to a mutable memory cell to be updated.
    Thunk(Weak<RefCell<Closure>>),
    /// The continuation of a primitive operation.
    Cont(
        OperationCont,
        usize,           /*callStack size*/
        Option<RawSpan>, /*position span of the operation*/
    ),
}

impl Marker {
    pub fn is_arg(&self) -> bool {
        match *self {
            Marker::Arg(_, _) => true,
            Marker::Thunk(_) => false,
            Marker::Cont(_, _, _) => false,
        }
    }

    pub fn is_thunk(&self) -> bool {
        match *self {
            Marker::Arg(_, _) => false,
            Marker::Thunk(_) => true,
            Marker::Cont(_, _, _) => false,
        }
    }

    pub fn is_cont(&self) -> bool {
        match *self {
            Marker::Arg(_, _) => false,
            Marker::Thunk(_) => false,
            Marker::Cont(_, _, _) => true,
        }
    }
}

/// The evaluation stack.
#[derive(Debug)]
pub struct Stack(Vec<Marker>);

impl IntoIterator for Stack {
    type Item = Marker;
    type IntoIter = ::std::vec::IntoIter<Marker>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Stack {
    pub fn new() -> Stack {
        Stack(Vec::new())
    }

    /// Count the number of consecutive elements satisfying `pred` from the top of the stack.
    fn count<P>(&self, pred: P) -> usize
    where
        P: Fn(&Marker) -> bool,
    {
        let mut count = 0;
        for marker in self.0.iter().rev() {
            if pred(marker) {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// Count the number of arguments at the top of the stack.
    pub fn count_args(&self) -> usize {
        Stack::count(self, Marker::is_arg)
    }

    pub fn push_arg(&mut self, arg: Closure, pos: Option<RawSpan>) {
        self.0.push(Marker::Arg(arg, pos))
    }

    pub fn push_thunk(&mut self, thunk: Weak<RefCell<Closure>>) {
        self.0.push(Marker::Thunk(thunk))
    }

    pub fn push_op_cont(&mut self, cont: OperationCont, len: usize, pos: Option<RawSpan>) {
        self.0.push(Marker::Cont(cont, len, pos))
    }

    pub fn pop_arg(&mut self) -> Option<(Closure, Option<RawSpan>)> {
        match self.0.pop() {
            Some(Marker::Arg(arg, pos)) => Some((arg, pos)),
            Some(m) => {
                self.0.push(m);
                None
            }
            _ => None,
        }
    }

    pub fn pop_thunk(&mut self) -> Option<Weak<RefCell<Closure>>> {
        match self.0.pop() {
            Some(Marker::Thunk(thunk)) => Some(thunk),
            Some(m) => {
                self.0.push(m);
                None
            }
            _ => None,
        }
    }

    pub fn pop_op_cont(&mut self) -> Option<(OperationCont, usize, Option<RawSpan>)> {
        match self.0.pop() {
            Some(Marker::Cont(cont, len, pos)) => Some((cont, len, pos)),
            Some(m) => {
                self.0.push(m);
                None
            }
            _ => None,
        }
    }

    /// Check if the top element is an argument.
    pub fn is_top_thunk(&self) -> bool {
        self.0.last().map(Marker::is_thunk).unwrap_or(false)
    }

    /// Check if the top element is an operation continuation.
    pub fn is_top_cont(&self) -> bool {
        self.0.last().map(Marker::is_cont).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::{Term, UnaryOp};
    use std::rc::Rc;

    impl Stack {
        /// Count the number of thunks at the top of the stack.
        pub fn count_thunks(&self) -> usize {
            Stack::count(self, Marker::is_thunk)
        }

        /// Count the number of operation continuation at the top of the stack.
        pub fn count_conts(&self) -> usize {
            Stack::count(self, Marker::is_cont)
        }
    }

    fn some_closure() -> Closure {
        Closure::atomic_closure(Term::Bool(true).into())
    }

    fn some_cont() -> OperationCont {
        OperationCont::Op1(UnaryOp::IsZero(), None)
    }

    fn some_arg_marker() -> Marker {
        Marker::Arg(some_closure(), None)
    }

    fn some_thunk_marker() -> Marker {
        let rc = Rc::new(RefCell::new(some_closure()));
        Marker::Thunk(Rc::downgrade(&rc))
    }

    fn some_cont_marker() -> Marker {
        Marker::Cont(some_cont(), 42, None)
    }

    #[test]
    fn marker_differentiates() {
        assert!(some_arg_marker().is_arg());
        assert!(some_thunk_marker().is_thunk());
        assert!(some_cont_marker().is_cont());
    }

    #[test]
    fn pushing_and_poping_args() {
        let mut s = Stack::new();
        assert_eq!(0, s.count_args());

        s.push_arg(some_closure(), None);
        s.push_arg(some_closure(), None);
        assert_eq!(2, s.count_args());
        assert_eq!(some_closure(), s.pop_arg().expect("Already checked").0);
        assert_eq!(1, s.count_args());
    }

    #[test]
    fn pushing_and_poping_thunks() {
        let mut s = Stack::new();
        assert_eq!(0, s.count_thunks());

        s.push_thunk(Rc::downgrade(&Rc::new(RefCell::new(some_closure()))));
        s.push_thunk(Rc::downgrade(&Rc::new(RefCell::new(some_closure()))));
        assert_eq!(2, s.count_thunks());
        s.pop_thunk().expect("Already checked");
        assert_eq!(1, s.count_thunks());
    }

    #[test]
    fn pushing_and_poping_conts() {
        let mut s = Stack::new();
        assert_eq!(0, s.count_conts());

        s.push_op_cont(some_cont(), 3, None);
        s.push_op_cont(some_cont(), 4, None);
        assert_eq!(2, s.count_conts());
        assert_eq!(
            (some_cont(), 4, None),
            s.pop_op_cont().expect("Already checked")
        );
        assert_eq!(1, s.count_conts());
    }
}
