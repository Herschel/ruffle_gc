use ruffle_gc::{GcContext, GcHeapRoot};

#[test]
fn test_gc() {
    let mut ctx = GcContext::new().unwrap();
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
