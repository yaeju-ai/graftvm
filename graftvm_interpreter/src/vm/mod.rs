use std::collections::HashMap;

use graftvm_bytecode::{Bytecode, Opcode};
use graftvm_liternal::Liternal;
use graftvm_window::{Window, WindowSlot};

mod arithmetic;
mod bitwise;
mod compare;
mod constant;
mod conversion;
mod window;

#[derive(Default)]
pub(super) struct VMState {
    pub cmp: bool,
}

pub struct VM {
    bytecode: Bytecode,
    pc: usize,
    pub(super) window_stack: Vec<Window>,
    pub(super) constant_pool: HashMap<usize, Liternal>,
    pub(super) state: VMState,
    /// Return-address stack for Call/Ret.
    return_stack: Vec<usize>,
    /// Argument stack for cross-window argument passing.
    arg_stack: Vec<Liternal>,
}

// ── Width-dispatch macros used by arithmetic/bitwise/compare modules ──

macro_rules! binop_width {
    ($self:expr, $dst:expr, $lhs:expr, $rhs:expr, $ty:expr, |$v:ident, $r:ident| $expr:expr) => {{
        let (val_l, val_r) = $self.read_two($lhs, $rhs)?;
        let result = match $ty {
            Width::I8 => {
                let $v = val_l.expect_int()?.expect_i8()?;
                let $r = val_r.expect_int()?.expect_i8()?;
                graftvm_liternal::Liternal::from($expr)
            }
            Width::I16 => {
                let $v = val_l.expect_int()?.expect_i16()?;
                let $r = val_r.expect_int()?.expect_i16()?;
                graftvm_liternal::Liternal::from($expr)
            }
            Width::I32 => {
                let $v = val_l.expect_int()?.expect_i32()?;
                let $r = val_r.expect_int()?.expect_i32()?;
                graftvm_liternal::Liternal::from($expr)
            }
            Width::I64 => {
                let $v = val_l.expect_int()?.expect_i64()?;
                let $r = val_r.expect_int()?.expect_i64()?;
                graftvm_liternal::Liternal::from($expr)
            }
            Width::U8 => {
                let $v = val_l.expect_uint()?.expect_u8()?;
                let $r = val_r.expect_uint()?.expect_u8()?;
                graftvm_liternal::Liternal::from($expr)
            }
            Width::U16 => {
                let $v = val_l.expect_uint()?.expect_u16()?;
                let $r = val_r.expect_uint()?.expect_u16()?;
                graftvm_liternal::Liternal::from($expr)
            }
            Width::U32 => {
                let $v = val_l.expect_uint()?.expect_u32()?;
                let $r = val_r.expect_uint()?.expect_u32()?;
                graftvm_liternal::Liternal::from($expr)
            }
            Width::U64 => {
                let $v = val_l.expect_uint()?.expect_u64()?;
                let $r = val_r.expect_uint()?.expect_u64()?;
                graftvm_liternal::Liternal::from($expr)
            }
            Width::F32 => {
                let $v = val_l.expect_float()?.expect_f32()?;
                let $r = val_r.expect_float()?.expect_f32()?;
                graftvm_liternal::Liternal::from($expr)
            }
            Width::F64 => {
                let $v = val_l.expect_float()?.expect_f64()?;
                let $r = val_r.expect_float()?.expect_f64()?;
                graftvm_liternal::Liternal::from($expr)
            }
        };
        *$self.slot_mut(($dst).slot) = Some(graftvm_window::WindowSlot::from(result));
        Ok(())
    }};
}

pub(super) use binop_width;

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        Self {
            bytecode,
            pc: 0,
            window_stack: vec![Window::new(0)],
            constant_pool: HashMap::new(),
            state: VMState::default(),
            return_stack: Vec::new(),
            arg_stack: Vec::new(),
        }
    }

    fn execute(&mut self, opcode: Opcode) -> Result<(), String> {
        match opcode {
            Opcode::Add { dst, lhs, rhs, ty } => self.add(dst, lhs, rhs, ty)?,
            Opcode::Sub { dst, lhs, rhs, ty } => self.sub(dst, lhs, rhs, ty)?,
            Opcode::Mul { dst, lhs, rhs, ty } => self.mul(dst, lhs, rhs, ty)?,
            Opcode::Div { dst, lhs, rhs, ty } => self.div(dst, lhs, rhs, ty)?,
            Opcode::Rem { dst, lhs, rhs, ty } => self.rem(dst, lhs, rhs, ty)?,
            Opcode::Neg { dst, src, ty } => self.neg(dst, src, ty)?,

            Opcode::Lt { dst, lhs, rhs, ty } => self.cmp_lt(dst, lhs, rhs, ty)?,
            Opcode::Le { dst, lhs, rhs, ty } => self.cmp_le(dst, lhs, rhs, ty)?,
            Opcode::Gt { dst, lhs, rhs, ty } => self.cmp_gt(dst, lhs, rhs, ty)?,
            Opcode::Ge { dst, lhs, rhs, ty } => self.cmp_ge(dst, lhs, rhs, ty)?,
            Opcode::Eq { lhs, rhs } => self.cmp_eq(lhs, rhs)?,
            Opcode::Neq { lhs, rhs } => self.cmp_neq(lhs, rhs)?,

            Opcode::And { dst, lhs, rhs, ty } => self.and(dst, lhs, rhs, ty)?,
            Opcode::Or { dst, lhs, rhs, ty } => self.or(dst, lhs, rhs, ty)?,
            Opcode::Xor { dst, lhs, rhs, ty } => self.xor(dst, lhs, rhs, ty)?,
            Opcode::Not { dst, src, ty } => self.not(dst, src, ty)?,

            Opcode::Shl { dst, lhs, rhs, ty } => self.shl(dst, lhs, rhs, ty)?,
            Opcode::Shr { dst, lhs, rhs, ty } => self.shr(dst, lhs, rhs, ty)?,

            Opcode::Extend { dst, src, signed } => self.extend(dst, src, signed)?,
            Opcode::Trunc { dst, src } => self.trunc(dst, src)?,
            Opcode::Reinterpret { dst, src } => self.reinterpret(dst, src)?,
            Opcode::Convert { dst, src, signed } => self.convert(dst, src, signed)?,

            Opcode::Jump(pc) => {
                self.pc = pc;
                return Ok(());
            }
            Opcode::Branch(pc) => {
                if self.state.cmp {
                    self.pc = pc;
                }
                return Ok(());
            }
            Opcode::Call(target) => {
                self.return_stack.push(self.pc + 1);
                self.pc = target;
                return Ok(());
            }
            Opcode::Ret => {
                if let Some(ret_addr) = self.return_stack.pop() {
                    self.pc = ret_addr;
                } else {
                    return Err("return stack empty".into());
                }
                return Ok(());
            }

            Opcode::PushArg { src } => {
                let val = self.read_one(src)?;
                self.arg_stack.push(val);
            }
            Opcode::PopArg { dst } => {
                let val = self.arg_stack.pop()
                    .ok_or_else(|| "argument stack empty".to_string())?;
                *self.slot_mut(dst.slot) = Some(WindowSlot::from(val));
            }

            Opcode::StoreData { index, data } => self.store_data(index, data),
            Opcode::LoadData { dst, index } => self.load_data(dst, index)?,

            Opcode::Move { dst, src } => {
                let val = self.slot_mut(src.slot).take();
                *self.slot_mut(dst.slot) = val;
            }
            Opcode::Copy { dst, src } => {
                let val = self.read_one(src)?.clone();
                *self.slot_mut(dst.slot) = Some(WindowSlot::from(val));
            }
            Opcode::Drop { src } => {
                *self.slot_mut(src.slot) = None;
            }

            Opcode::Enter => self.enter_window()?,
            Opcode::Exit => self.exit_window()?,
        }

        self.pc += 1;
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), String> {
        let opcode = self.bytecode[self.pc].clone();
        self.execute(opcode)
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.pc < self.bytecode.len() {
            self.step()?;
        }
        Ok(())
    }

    /// Run with full instruction trace printed to stdout.
    pub fn run_debug(&mut self) -> Result<(), String> {
        while self.pc < self.bytecode.len() {
            let op = &self.bytecode[self.pc];
            println!("  [{:>3}] {:?}", self.pc, op);
            self.step()?;
        }
        Ok(())
    }

    /// Dump the final state of all windows and constants.
    pub fn dump_state(&self) {
        println!(";; ── VM state ──");
        for (_i, w) in self.window_stack.iter().enumerate() {
            println!(";; window {} ({} slots):", w.id, w.slots.len());
            for (j, slot) in w.slots.iter().enumerate() {
                if let Some(s) = slot {
                    println!(";;   slot {}: {:?}", j, s.data);
                }
            }
        }
        println!(";; cmp flag: {}", self.state.cmp);
        if !self.arg_stack.is_empty() {
            println!(";; arg stack ({} items): {:?}", self.arg_stack.len(), self.arg_stack);
        }
    }
}

// ── Slot read helpers (owned, not borrowed) ──

impl VM {
    /// Read one slot's data as an owned `Liternal`.
    pub(super) fn read_one(&self, addr: graftvm_bytecode::Addr) -> Result<Liternal, String> {
        let w = self.current_window();
        w.slots
            .get(addr.slot)
            .and_then(|s| s.as_ref())
            .map(|s| s.data.clone())
            .ok_or_else(|| format!("slot {} empty or out of bounds", addr.slot))
    }

    /// Read two slots' data as owned `Liternal` values.
    pub(super) fn read_two(
        &self,
        lhs: graftvm_bytecode::Addr,
        rhs: graftvm_bytecode::Addr,
    ) -> Result<(Liternal, Liternal), String> {
        let w = self.current_window();
        let l = w
            .slots
            .get(lhs.slot)
            .and_then(|s| s.as_ref())
            .map(|s| s.data.clone())
            .ok_or_else(|| format!("slot {} empty", lhs.slot))?;
        let r = w
            .slots
            .get(rhs.slot)
            .and_then(|s| s.as_ref())
            .map(|s| s.data.clone())
            .ok_or_else(|| format!("slot {} empty", rhs.slot))?;
        Ok((l, r))
    }

    /// Mutable reference to a slot (resizes vec as needed).
    pub(super) fn slot_mut(&mut self, slot: usize) -> &mut Option<WindowSlot> {
        let w = self.current_window_mut();
        if slot >= w.slots.len() {
            w.slots.resize(slot + 1, None);
        }
        &mut w.slots[slot]
    }

    // Helpers for window access
    pub(super) fn current_window(&self) -> &Window {
        let len = self.window_stack.len();
        &self.window_stack[len - 1]
    }

    pub(super) fn current_window_mut(&mut self) -> &mut Window {
        let len = self.window_stack.len();
        &mut self.window_stack[len - 1]
    }
}
