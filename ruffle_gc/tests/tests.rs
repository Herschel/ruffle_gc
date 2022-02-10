use ruffle_gc::{new_gc_context, GcHeapRoot};

#[test]
fn test_gc() {
    new_gc_context!(ctx);
    let object = GcHeapRoot::new(ctx.allocate("Test".to_string()));
    ctx.collect();
    assert_eq!(*object.borrow(&ctx), "Test");
}

#[test]
fn compile_fails() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fails/allocate.rs");
    t.compile_fail("tests/compile_fails/borrow_mut.rs");
}
