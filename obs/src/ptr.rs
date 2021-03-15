#![allow(dead_code)]

use std::mem;

#[inline(always)]
pub unsafe fn assert_as_ref<'a, CT, T>(ptr: *mut CT) -> Option<&'a T> {
    let data: *const T = mem::transmute(ptr);
    data.as_ref()
}

#[inline(always)]
pub unsafe fn assert_as_mut<'a, CT, T>(ptr: *mut CT) -> Option<&'a mut T> {
    let data: *mut T = mem::transmute(ptr);
    data.as_mut()
}

#[inline(always)]
pub unsafe fn assert_ref<'a, CT, T>(ptr: *mut CT) -> &'a T {
    assert_as_ref(ptr).unwrap()
}

#[inline(always)]
pub unsafe fn assert_mut<'a, CT, T>(ptr: *mut CT) -> &'a mut T {
    assert_as_mut(ptr).unwrap()
}
