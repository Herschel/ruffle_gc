use crate::{GcContext, GcLifetime, GcVtbl, Trace};
use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    ptr,
};

#[repr(C)]
pub struct GcRoot<T> {
    pub(crate) vtbl: GcVtbl,
    pub(crate) next: *mut GcRootData,
    pub(crate) prev: *mut GcRootData,
    pub(crate) value: UnsafeCell<T>,
}

pub type GcRootData = GcRoot<()>;

impl<T> GcRoot<T> {
    pub unsafe fn new<'a>(value: T) -> GcRoot<T::Aged>
    where
        T: GcLifetime<'a> + Trace,
    {
        GcRoot {
            vtbl: T::vtbl(),
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
            value: UnsafeCell::new(value.change_lifetime()),
        }
    }

    pub fn pin<'a>(&'a mut self) -> &'a T
    where
        T: GcLifetime<'a>,
        T: Trace,
    {
        unsafe {
            let ptr = self as *mut GcRoot<_> as *mut GcRootData;
            GcContext::get().insert_root(ptr);
            &*self.value.get()
        }
    }
}

impl<T> Drop for GcRoot<T> {
    fn drop(&mut self) {
        unsafe {
            GcContext::get().remove_root(self as *mut GcRoot<_> as *const GcRootData);
        }
    }
}

#[repr(transparent)]
pub struct GcHeapRoot<T>(pub(crate) Box<GcRoot<T>>);

impl<T> GcHeapRoot<T> {
    pub fn new<'a>(value: T) -> GcHeapRoot<T::Aged>
    where
        T: GcLifetime<'a> + Trace,
    {
        unsafe {
            let root_data = GcRoot::new(value);
            let mut boxed = Box::new(root_data);
            let ptr = &mut *boxed as *mut GcRoot<_> as *mut GcRootData;
            GcContext::get().insert_root(ptr);
            GcHeapRoot(boxed)
        }
    }
}

impl<'a, T> Deref for GcHeapRoot<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.deref().value.get() }
    }
}

impl<'a, T> DerefMut for GcHeapRoot<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.deref().value.get() }
    }
}
