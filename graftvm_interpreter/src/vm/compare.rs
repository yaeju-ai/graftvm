use graftvm_bytecode::{Addr, Width};
use graftvm_liternal::Liternal;
use graftvm_window::WindowSlot;

use crate::vm::VM;

impl VM {
    pub(crate) fn cmp_lt(&mut self, dst: Addr, lhs: Addr, rhs: Addr, ty: Width) -> Result<(), String> {
        let (vl, vr) = self.read_two(lhs, rhs)?;
        let ok = match ty {
            Width::I8 => vl.expect_int()?.expect_i8()? < vr.expect_int()?.expect_i8()?,
            Width::I16 => vl.expect_int()?.expect_i16()? < vr.expect_int()?.expect_i16()?,
            Width::I32 => vl.expect_int()?.expect_i32()? < vr.expect_int()?.expect_i32()?,
            Width::I64 => vl.expect_int()?.expect_i64()? < vr.expect_int()?.expect_i64()?,
            Width::U8 => vl.expect_uint()?.expect_u8()? < vr.expect_uint()?.expect_u8()?,
            Width::U16 => vl.expect_uint()?.expect_u16()? < vr.expect_uint()?.expect_u16()?,
            Width::U32 => vl.expect_uint()?.expect_u32()? < vr.expect_uint()?.expect_u32()?,
            Width::U64 => vl.expect_uint()?.expect_u64()? < vr.expect_uint()?.expect_u64()?,
            Width::F32 => vl.expect_float()?.expect_f32()? < vr.expect_float()?.expect_f32()?,
            Width::F64 => vl.expect_float()?.expect_f64()? < vr.expect_float()?.expect_f64()?,
        };
        *self.slot_mut(dst.slot) = Some(WindowSlot::from(Liternal::from(if ok { 1i64 } else { 0i64 })));
        Ok(())
    }

    pub(crate) fn cmp_le(&mut self, dst: Addr, lhs: Addr, rhs: Addr, ty: Width) -> Result<(), String> {
        let (vl, vr) = self.read_two(lhs, rhs)?;
        let ok = match ty {
            Width::I8 => vl.expect_int()?.expect_i8()? <= vr.expect_int()?.expect_i8()?,
            Width::I16 => vl.expect_int()?.expect_i16()? <= vr.expect_int()?.expect_i16()?,
            Width::I32 => vl.expect_int()?.expect_i32()? <= vr.expect_int()?.expect_i32()?,
            Width::I64 => vl.expect_int()?.expect_i64()? <= vr.expect_int()?.expect_i64()?,
            Width::U8 => vl.expect_uint()?.expect_u8()? <= vr.expect_uint()?.expect_u8()?,
            Width::U16 => vl.expect_uint()?.expect_u16()? <= vr.expect_uint()?.expect_u16()?,
            Width::U32 => vl.expect_uint()?.expect_u32()? <= vr.expect_uint()?.expect_u32()?,
            Width::U64 => vl.expect_uint()?.expect_u64()? <= vr.expect_uint()?.expect_u64()?,
            Width::F32 => vl.expect_float()?.expect_f32()? <= vr.expect_float()?.expect_f32()?,
            Width::F64 => vl.expect_float()?.expect_f64()? <= vr.expect_float()?.expect_f64()?,
        };
        *self.slot_mut(dst.slot) = Some(WindowSlot::from(Liternal::from(if ok { 1i64 } else { 0i64 })));
        Ok(())
    }

    pub(crate) fn cmp_gt(&mut self, dst: Addr, lhs: Addr, rhs: Addr, ty: Width) -> Result<(), String> {
        let (vl, vr) = self.read_two(lhs, rhs)?;
        let ok = match ty {
            Width::I8 => vl.expect_int()?.expect_i8()? > vr.expect_int()?.expect_i8()?,
            Width::I16 => vl.expect_int()?.expect_i16()? > vr.expect_int()?.expect_i16()?,
            Width::I32 => vl.expect_int()?.expect_i32()? > vr.expect_int()?.expect_i32()?,
            Width::I64 => vl.expect_int()?.expect_i64()? > vr.expect_int()?.expect_i64()?,
            Width::U8 => vl.expect_uint()?.expect_u8()? > vr.expect_uint()?.expect_u8()?,
            Width::U16 => vl.expect_uint()?.expect_u16()? > vr.expect_uint()?.expect_u16()?,
            Width::U32 => vl.expect_uint()?.expect_u32()? > vr.expect_uint()?.expect_u32()?,
            Width::U64 => vl.expect_uint()?.expect_u64()? > vr.expect_uint()?.expect_u64()?,
            Width::F32 => vl.expect_float()?.expect_f32()? > vr.expect_float()?.expect_f32()?,
            Width::F64 => vl.expect_float()?.expect_f64()? > vr.expect_float()?.expect_f64()?,
        };
        *self.slot_mut(dst.slot) = Some(WindowSlot::from(Liternal::from(if ok { 1i64 } else { 0i64 })));
        Ok(())
    }

    pub(crate) fn cmp_ge(&mut self, dst: Addr, lhs: Addr, rhs: Addr, ty: Width) -> Result<(), String> {
        let (vl, vr) = self.read_two(lhs, rhs)?;
        let ok = match ty {
            Width::I8 => vl.expect_int()?.expect_i8()? >= vr.expect_int()?.expect_i8()?,
            Width::I16 => vl.expect_int()?.expect_i16()? >= vr.expect_int()?.expect_i16()?,
            Width::I32 => vl.expect_int()?.expect_i32()? >= vr.expect_int()?.expect_i32()?,
            Width::I64 => vl.expect_int()?.expect_i64()? >= vr.expect_int()?.expect_i64()?,
            Width::U8 => vl.expect_uint()?.expect_u8()? >= vr.expect_uint()?.expect_u8()?,
            Width::U16 => vl.expect_uint()?.expect_u16()? >= vr.expect_uint()?.expect_u16()?,
            Width::U32 => vl.expect_uint()?.expect_u32()? >= vr.expect_uint()?.expect_u32()?,
            Width::U64 => vl.expect_uint()?.expect_u64()? >= vr.expect_uint()?.expect_u64()?,
            Width::F32 => vl.expect_float()?.expect_f32()? >= vr.expect_float()?.expect_f32()?,
            Width::F64 => vl.expect_float()?.expect_f64()? >= vr.expect_float()?.expect_f64()?,
        };
        *self.slot_mut(dst.slot) = Some(WindowSlot::from(Liternal::from(if ok { 1i64 } else { 0i64 })));
        Ok(())
    }

    pub(crate) fn cmp_eq(&mut self, lhs: Addr, rhs: Addr) -> Result<(), String> {
        let (vl, vr) = self.read_two(lhs, rhs)?;
        self.state.cmp = vl == vr;
        Ok(())
    }

    pub(crate) fn cmp_neq(&mut self, lhs: Addr, rhs: Addr) -> Result<(), String> {
        self.cmp_eq(lhs, rhs)?;
        self.state.cmp = !self.state.cmp;
        Ok(())
    }
}
