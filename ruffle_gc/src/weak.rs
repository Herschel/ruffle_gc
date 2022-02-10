use crate::{Gc, GcContext, GcData, GcLifetime, Invariant, Trace};
use std::marker::PhantomData;

/// A weak pointer to memory managed by the garbage collector.
///
/// A `GcWeak` pointer will not prevent the pointed-to data from being collected. Therefore,
/// attempting to borrow the data via `borrow` or `upgrade` may return `None` if the data
/// has been collecting.
#[repr(transparent)]
pub struct GcWeak<'a, 'gc, T> {
    pub(crate) id: WeakId,
    pub(crate) _invariant: Invariant<'gc>,
    pub(crate) _phantom: PhantomData<&'a T>,
}

pub(crate) type WeakId = generational_arena::Index;

impl<'a, 'gc, T> GcWeak<'a, 'gc, T> {
    /// Attempts to upgrade the weak pointer to a `Gc`.
    ///
    /// This requires immutable to the `GcContext` to ensure that the inner value does not get
    /// collected while the `Gc` pointer is not rooted. Returns `None` if the inner value has
    /// already been collected.
    pub fn upgrade(self, ctx: &'a GcContext<'gc>) -> Option<Gc<'a, 'gc, T>> {
        ctx.get_weak(self)
    }

    /// Attempts to borrow the inner value pointed to by the weak pointer.
    ///
    /// This requires immutable to the `GcContext` to ensure that the inner value does not get
    /// collected until the borrow is complete. Returns `None` if the inner value has already been
    /// collected.
    pub fn borrow<'b>(self, ctx: &'b GcContext<'gc>) -> Option<&'b T::Aged>
    where
        T: GcLifetime<'b>,
        'a: 'b,
    {
        ctx.get_weak(self)
            .map(|gc| unsafe { &*(*(gc.ptr as *mut GcData<T::Aged>)).value.get() })
    }

    /// Attempts to mutably borrow the inner value pointed to by the weak pointer.
    /// This requires mutable to the `GcContext` to ensure that no other managed data can be
    /// mutated until the borrow is complete. Returns `None` if the inner value has already been
    /// collected.
    pub fn borrow_mut<'b>(self, ctx: &'b mut GcContext<'gc>) -> Option<&'b mut T::Aged>
    where
        T: GcLifetime<'b>,
        'a: 'b,
    {
        ctx.get_weak(self)
            .map(|gc| unsafe { &mut *(*(gc.ptr as *mut GcData<T::Aged>)).value.get() })
    }
}

impl<'a, 'gc, T> Clone for GcWeak<'a, 'gc, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, 'gc, T> Copy for GcWeak<'a, 'gc, T> {}

unsafe impl<'a, 'b, 'gc, T> GcLifetime<'a> for GcWeak<'b, 'gc, T>
where
    T: 'a + GcLifetime<'a>,
{
    type Aged = GcWeak<'a, 'gc, T::Aged>;
}

unsafe impl<'a, 'gc, T> Trace for GcWeak<'a, 'gc, T> {
    unsafe fn trace(&self, _ctx: &mut GcContext) {
        // Noop; weak pointers don't keep other managed data alive.
    }
}
