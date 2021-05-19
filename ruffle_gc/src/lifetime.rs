use crate::Gc;
use std::{cell::*, collections::*, mem, num::*};

pub unsafe trait GcLifetime<'a> {
    type Aged;

    unsafe fn change_lifetime(self) -> Self::Aged
    where
        Self: Sized,
    {
        let result = mem::transmute_copy(&self);
        mem::forget(self);
        result
    }
}

unsafe impl GcLifetime<'_> for u8 {
    type Aged = u8;
}
unsafe impl GcLifetime<'_> for u16 {
    type Aged = u16;
}
unsafe impl GcLifetime<'_> for u32 {
    type Aged = u32;
}
unsafe impl GcLifetime<'_> for u64 {
    type Aged = u64;
}
unsafe impl GcLifetime<'_> for u128 {
    type Aged = u128;
}
unsafe impl GcLifetime<'_> for usize {
    type Aged = usize;
}
unsafe impl GcLifetime<'_> for NonZeroU8 {
    type Aged = NonZeroU8;
}
unsafe impl GcLifetime<'_> for NonZeroU16 {
    type Aged = NonZeroU16;
}
unsafe impl GcLifetime<'_> for NonZeroU32 {
    type Aged = NonZeroU32;
}
unsafe impl GcLifetime<'_> for NonZeroU64 {
    type Aged = NonZeroU64;
}
unsafe impl GcLifetime<'_> for NonZeroU128 {
    type Aged = NonZeroU128;
}
unsafe impl GcLifetime<'_> for NonZeroUsize {
    type Aged = NonZeroUsize;
}
unsafe impl GcLifetime<'_> for i8 {
    type Aged = i8;
}
unsafe impl GcLifetime<'_> for i16 {
    type Aged = i16;
}
unsafe impl GcLifetime<'_> for i32 {
    type Aged = i32;
}
unsafe impl GcLifetime<'_> for i64 {
    type Aged = i64;
}
unsafe impl GcLifetime<'_> for i128 {
    type Aged = i128;
}
unsafe impl GcLifetime<'_> for isize {
    type Aged = isize;
}
unsafe impl GcLifetime<'_> for NonZeroI8 {
    type Aged = NonZeroI8;
}
unsafe impl GcLifetime<'_> for NonZeroI16 {
    type Aged = NonZeroI16;
}
unsafe impl GcLifetime<'_> for NonZeroI32 {
    type Aged = NonZeroI32;
}
unsafe impl GcLifetime<'_> for NonZeroI64 {
    type Aged = NonZeroI64;
}
unsafe impl GcLifetime<'_> for NonZeroI128 {
    type Aged = NonZeroI128;
}
unsafe impl GcLifetime<'_> for NonZeroIsize {
    type Aged = NonZeroIsize;
}
unsafe impl GcLifetime<'_> for f32 {
    type Aged = f32;
}
unsafe impl GcLifetime<'_> for f64 {
    type Aged = f64;
}
unsafe impl GcLifetime<'_> for bool {
    type Aged = bool;
}
unsafe impl GcLifetime<'_> for char {
    type Aged = char;
}
unsafe impl GcLifetime<'_> for String {
    type Aged = String;
}
unsafe impl GcLifetime<'_> for () {
    type Aged = ();
}

unsafe impl<'a, 'b, T> GcLifetime<'a> for Gc<'b, T>
where
    T: GcLifetime<'a> + 'a,
{
    type Aged = Gc<'a, T::Aged>;
}

unsafe impl<'a, T> GcLifetime<'a> for Option<T>
where
    T: GcLifetime<'a>,
{
    type Aged = Option<T::Aged>;
}

unsafe impl<'a, T, E> GcLifetime<'a> for Result<T, E>
where
    T: GcLifetime<'a>,
    E: GcLifetime<'a>,
{
    type Aged = Result<T::Aged, E::Aged>;
}

unsafe impl<'a, T> GcLifetime<'a> for Cell<T>
where
    T: GcLifetime<'a>,
{
    type Aged = Cell<T::Aged>;
}

unsafe impl<'a, T> GcLifetime<'a> for RefCell<T>
where
    T: GcLifetime<'a>,
{
    type Aged = RefCell<T::Aged>;
}

unsafe impl<'a, 'b, T> GcLifetime<'a> for BinaryHeap<T>
where
    T: GcLifetime<'a>,
{
    type Aged = BinaryHeap<T::Aged>;
}

unsafe impl<'a, 'b, K, V> GcLifetime<'a> for BTreeMap<K, V>
where
    K: GcLifetime<'a>,
    V: GcLifetime<'a>,
{
    type Aged = BTreeMap<K::Aged, V::Aged>;
}

unsafe impl<'a, 'b, T> GcLifetime<'a> for BTreeSet<T>
where
    T: GcLifetime<'a>,
{
    type Aged = BTreeSet<T::Aged>;
}

unsafe impl<'a, 'b, K, V> GcLifetime<'a> for HashMap<K, V>
where
    K: GcLifetime<'a>,
    V: GcLifetime<'a>,
{
    type Aged = HashMap<K::Aged, V::Aged>;
}

unsafe impl<'a, 'b, T> GcLifetime<'a> for HashSet<T>
where
    T: GcLifetime<'a>,
{
    type Aged = HashSet<T::Aged>;
}

unsafe impl<'a, 'b, T> GcLifetime<'a> for LinkedList<T>
where
    T: GcLifetime<'a>,
{
    type Aged = LinkedList<T::Aged>;
}

unsafe impl<'a, 'b, T> GcLifetime<'a> for Vec<T>
where
    T: GcLifetime<'a>,
{
    type Aged = Vec<T::Aged>;
}

unsafe impl<'a, 'b, T> GcLifetime<'a> for VecDeque<T>
where
    T: GcLifetime<'a>,
{
    type Aged = VecDeque<T::Aged>;
}
