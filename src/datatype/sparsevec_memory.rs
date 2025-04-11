use std::{marker::PhantomData, ptr::NonNull};

use pgrx::pgrx_sql_entity_graph::metadata::ArgumentError;
use pgrx::pgrx_sql_entity_graph::metadata::Returns;
use pgrx::pgrx_sql_entity_graph::metadata::ReturnsError;
use pgrx::pgrx_sql_entity_graph::metadata::SqlMapping;
use pgrx::pgrx_sql_entity_graph::metadata::SqlTranslatable;
use pgrx::{
    datum::UnboxDatum,
    pg_sys::{Datum, Oid},
    FromDatum, IntoDatum,
};

use crate::datatype::MAX_NNZ;

use super::sparsevec::SparsevecBorrowed;

#[repr(C, align(8))]
struct SparsevecHeader {
    varlena: u32,
    dim: u32,
    nnz: u32,
    unused: u32,
    indices: [u32; 0],
}

impl SparsevecHeader {
    fn size_of(nnz: usize) -> usize {
        if nnz > MAX_NNZ {
            panic!("sparsevec is too large");
        }
        size_of::<Self>() + 8 * nnz
    }
    fn indexes(&self) -> &[u32] {
        let ptr = self.indices.as_ptr();
        unsafe { std::slice::from_raw_parts(ptr, self.nnz as usize) }
    }
    fn values(&self) -> &[f32] {
        let nnz = self.nnz as usize;
        unsafe {
            let ptr = self.indices.as_ptr().add(nnz).cast();
            std::slice::from_raw_parts(ptr, nnz)
        }
    }
    unsafe fn as_borrowed<'a>(this: NonNull<Self>) -> SparsevecBorrowed<'a> {
        unsafe {
            let this = this.as_ref();
            SparsevecBorrowed::new_unchecked(this.dim, this.indexes(), this.values())
        }
    }
}

pub struct SparsevecInput<'a>(NonNull<SparsevecHeader>, PhantomData<&'a ()>, bool);

impl SparsevecInput<'_> {
    unsafe fn from_ptr(p: NonNull<SparsevecHeader>) -> Self {
        let q = unsafe {
            NonNull::new(pgrx::pg_sys::pg_detoast_datum(p.as_ptr().cast()).cast()).unwrap()
        };
        SparsevecInput(q, PhantomData, p != q)
    }
    pub fn as_borrowed(&self) -> SparsevecBorrowed<'_> {
        unsafe { SparsevecHeader::as_borrowed(self.0) }
    }
}

impl Drop for SparsevecInput<'_> {
    fn drop(&mut self) {
        if self.2 {
            unsafe {
                pgrx::pg_sys::pfree(self.0.as_ptr().cast());
            }
        }
    }
}

pub struct SparsevecOutput(NonNull<SparsevecHeader>);

impl SparsevecOutput {
    unsafe fn from_ptr(p: NonNull<SparsevecHeader>) -> Self {
        let q = unsafe {
            NonNull::new(pgrx::pg_sys::pg_detoast_datum_copy(p.as_ptr().cast()).cast()).unwrap()
        };
        Self(q)
    }
    pub fn new(vector: SparsevecBorrowed<'_>) -> Self {
        unsafe {
            let nnz = vector.len();
            let size = SparsevecHeader::size_of(nnz);

            let ptr = pgrx::pg_sys::palloc0(size) as *mut SparsevecHeader;
            (&raw mut (*ptr).varlena).write((size << 2) as u32);
            (&raw mut (*ptr).dim).write(vector.dims() as _);
            (&raw mut (*ptr).nnz).write(nnz as _);
            (&raw mut (*ptr).unused).write(0);
            std::ptr::copy_nonoverlapping(
                vector.indexes().as_ptr(),
                (*ptr).indices.as_mut_ptr(),
                nnz,
            );
            std::ptr::copy_nonoverlapping(
                vector.values().as_ptr(),
                (*ptr).indices.as_mut_ptr().add(nnz).cast(),
                nnz,
            );
            Self(NonNull::new(ptr).unwrap())
        }
    }
    pub fn as_borrowed(&self) -> SparsevecBorrowed<'_> {
        unsafe { SparsevecHeader::as_borrowed(self.0) }
    }
    fn into_raw(self) -> *mut SparsevecHeader {
        let ptr = self.0.as_ptr();
        std::mem::forget(self);
        ptr
    }
}

impl Drop for SparsevecOutput {
    fn drop(&mut self) {
        unsafe {
            pgrx::pg_sys::pfree(self.0.as_ptr().cast());
        }
    }
}

// FromDatum

impl FromDatum for SparsevecInput<'_> {
    unsafe fn from_polymorphic_datum(datum: Datum, is_null: bool, _typoid: Oid) -> Option<Self> {
        if is_null {
            None
        } else {
            let ptr = NonNull::new(datum.cast_mut_ptr()).unwrap();
            unsafe { Some(Self::from_ptr(ptr)) }
        }
    }
}

impl FromDatum for SparsevecOutput {
    unsafe fn from_polymorphic_datum(datum: Datum, is_null: bool, _typoid: Oid) -> Option<Self> {
        if is_null {
            None
        } else {
            let ptr = NonNull::new(datum.cast_mut_ptr()).unwrap();
            unsafe { Some(Self::from_ptr(ptr)) }
        }
    }
}

// IntoDatum

impl IntoDatum for SparsevecOutput {
    fn into_datum(self) -> Option<Datum> {
        Some(Datum::from(self.into_raw()))
    }

    fn type_oid() -> Oid {
        Oid::INVALID
    }

    fn is_compatible_with(_: Oid) -> bool {
        true
    }
}

// UnboxDatum

unsafe impl<'a> UnboxDatum for SparsevecInput<'a> {
    type As<'src>
        = SparsevecInput<'src>
    where
        'a: 'src;
    #[inline]
    unsafe fn unbox<'src>(datum: pgrx::datum::Datum<'src>) -> Self::As<'src>
    where
        Self: 'src,
    {
        let datum = datum.sans_lifetime();
        let ptr = NonNull::new(datum.cast_mut_ptr()).unwrap();
        unsafe { Self::from_ptr(ptr) }
    }
}

unsafe impl UnboxDatum for SparsevecOutput {
    type As<'src> = SparsevecOutput;
    #[inline]
    unsafe fn unbox<'src>(datum: pgrx::datum::Datum<'src>) -> Self::As<'src>
    where
        Self: 'src,
    {
        let datum = datum.sans_lifetime();
        let ptr = NonNull::new(datum.cast_mut_ptr()).unwrap();
        unsafe { Self::from_ptr(ptr) }
    }
}

// SqlTranslatable

unsafe impl SqlTranslatable for SparsevecInput<'_> {
    fn argument_sql() -> Result<SqlMapping, ArgumentError> {
        Ok(SqlMapping::As(String::from("sparsevec")))
    }
    fn return_sql() -> Result<Returns, ReturnsError> {
        Ok(Returns::One(SqlMapping::As(String::from("sparsevec"))))
    }
}

unsafe impl SqlTranslatable for SparsevecOutput {
    fn argument_sql() -> Result<SqlMapping, ArgumentError> {
        Ok(SqlMapping::As(String::from("sparsevec")))
    }
    fn return_sql() -> Result<Returns, ReturnsError> {
        Ok(Returns::One(SqlMapping::As(String::from("sparsevec"))))
    }
}

// ArgAbi

unsafe impl<'fcx> pgrx::callconv::ArgAbi<'fcx> for SparsevecInput<'fcx> {
    unsafe fn unbox_arg_unchecked(arg: pgrx::callconv::Arg<'_, 'fcx>) -> Self {
        let index = arg.index();
        unsafe {
            arg.unbox_arg_using_from_datum()
                .unwrap_or_else(|| panic!("argument {index} must not be null"))
        }
    }
}

// BoxAbi

unsafe impl pgrx::callconv::BoxRet for SparsevecOutput {
    unsafe fn box_into<'fcx>(
        self,
        fcinfo: &mut pgrx::callconv::FcInfo<'fcx>,
    ) -> pgrx::datum::Datum<'fcx> {
        match self.into_datum() {
            Some(datum) => unsafe { fcinfo.return_raw_datum(datum) },
            None => fcinfo.return_null(),
        }
    }
}
