#![allow(unused_variables)]

use ruffle_gc::{Gc, GcContext, GcHeapRoot};

fn main() {
    let mut ctx = GcContext::new().unwrap();

    let root1 = GcHeapRoot::new(Root::new(&mut ctx, "Testing1"));
    let root2 = GcHeapRoot::new(Root::new(&mut ctx, "Testing2"));

    // Okay: shared borrows of GcContext
    let s1 = root1.0.borrow(&ctx);
    let s2 = root2.0.borrow(&ctx);

    // Error: Attempt to mutably borrow context while above shared borrows still exist.
    let s2 = root2.0.borrow_mut(&mut ctx);

    println!("{}", s1);
}

#[derive(Gc)]
struct Root<'a>(Gc<'a, String>);

impl<'a> Root<'a> {
    fn new(ctx: &'a mut GcContext, s: impl Into<String>) -> Self {
        Self(ctx.allocate(s.into()))
    }
}
