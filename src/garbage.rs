use std::cell::RefCell;
use std::collections::VecDeque;
use std::mem::ManuallyDrop;

use crate::expression::Expression;

thread_local! {
    static GARBAGE: RefCell<VecDeque<Expression>> = RefCell::new(VecDeque::new());
}

pub fn dispose(expr: Expression) {
    let expr = ManuallyDrop::new(expr);
    let _ = GARBAGE.try_with(|garbage| {
        garbage.borrow_mut().push_back(ManuallyDrop::into_inner(expr));
    });
}

pub fn collect() {
    while let Some(expr) = GARBAGE.with(|garbage| garbage.borrow_mut().pop_front()) {
        drop(expr);
    }
}
