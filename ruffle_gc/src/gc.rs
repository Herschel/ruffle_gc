use crate::{GcContext, GcLifetime, GcWeak, Invariant, Trace, WeakId};
use bitflags::bitflags;
use std::{
    cell::UnsafeCell,
    fmt::{self, Debug},
    marker::PhantomData,
};

/// A pointer to garbage-collected memory.
#[repr(transparent)]
pub struct Gc<'a, 'gc, T> {
    pub(crate) ptr: GcDataPtr,
    pub(crate) _invariant: Invariant<'gc>,
    pub(crate) _phantom: PhantomData<&'a T>,
}

impl<'a, 'gc, T> Clone for Gc<'a, 'gc, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, 'gc, T> Copy for Gc<'a, 'gc, T> {}

impl<'a, 'gc, T> Gc<'a, 'gc, T> {
    pub fn get<'b>(self, _: &'b GcContext) -> T::Aged
    where
        T: GcLifetime<'b>,
        T::Aged: Copy,
        'a: 'b,
    {
        unsafe { *(self.ptr as *mut T::Aged) }
    }

    /// Immutably borrows the inner value pointed to by this pointer.
    ///
    /// This requires immutable access to the `GcContext` to ensure that no other managed data is
    /// mutated for the duration of the borrow.
    pub fn borrow<'b>(self, _: &'b GcContext) -> &'b T::Aged
    where
        T: GcLifetime<'b>,
        'a: 'b,
    {
        unsafe { &*(*(self.ptr as *mut GcData<T::Aged>)).value.get() }
    }

    /// Mutably borrows the inner value pointed to by this pointer.
    ///
    /// This requires mutable access to the `GcContext` to ensure that no other managed data can
    /// be accessed for the duration of the borrow.
    pub fn borrow_mut<'b>(self, _: &'b mut GcContext) -> &'b mut T::Aged
    where
        T: GcLifetime<'b>,
        'a: 'b,
    {
        unsafe { &mut *(*(self.ptr as *mut GcData<T::Aged>)).value.get() }
    }

    pub fn downgrade(self, ctx: &GcContext) -> GcWeak<'a, 'gc, T> {
        let weak = unsafe { (*self.ptr).weak };
        let id = if let Some(id) = weak {
            id
        } else {
            ctx.add_weak(self.ptr)
        };
        GcWeak {
            id,
            _invariant: self._invariant,
            _phantom: Default::default(),
        }
    }

    /// Returns `true` if this pointer points to the same value as `other`.
    pub fn ptr_eq(self, other: Gc<T>) -> bool {
        self.as_ptr() == other.as_ptr()
    }

    /// Returns a pointer to the underlying value.
    pub fn as_ptr(self) -> *const T {
        self.ptr as *mut T as *const T
    }
}

impl<'a, 'gc, T> Debug for Gc<'a, 'gc, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Gc")
            .field("ptr", unsafe { &(*self.ptr).value })
            .finish()
    }
}

unsafe impl<'a, 'gc, T: Trace> Trace for Gc<'a, 'gc, T> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        ctx.trace(self.ptr);
    }
}

bitflags! {
    /// Flags stored into the header of a garbage collected allocation.
    pub(crate) struct GcFlags: u8 {
        const WHITE  = 0b00;
        const GRAY   = 0b01;
        const BLACK  = 0b10;
        const COLOR_MASK = 0b11;

        const NEEDS_TRACE = 0b100;
    }
}

pub(crate) type GcDataPtr = *mut GcData<()>;

#[repr(C)]
pub struct GcData<T> {
    pub(crate) vtbl: GcVtbl,
    pub(crate) flags: GcFlags,
    pub(crate) weak: Option<WeakId>,
    pub(crate) next: GcDataPtr,
    pub(crate) value: UnsafeCell<T>,
}

/// The virtual method table stored with garbage collected data.
#[repr(C)]
pub struct GcVtbl {
    pub(crate) trace: unsafe fn(*const (), &mut GcContext),
    pub(crate) dealloc: unsafe fn(*mut ()),
}
