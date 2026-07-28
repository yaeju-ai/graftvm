//! High-level IR builder for GraftVM.
//!
//! Provides variable binding, constant pooling, control-flow labels, and
//! function scopes on top of raw `Opcode` — so you never touch `Addr` directly.
//!
//! # Example
//!
//! ```rust
//! use graftvm_ir::IrBuilder;
//! use graftvm_bytecode::Width;
//!
//! let mut ir = IrBuilder::new();
//!
//! let x = ir.var("x", Width::I32);
//! let y = ir.var("y", Width::I32);
//! let z = ir.var("z", Width::I32);
//!
//! let ten = ir.i32("ten", 10);
//!
//! ir.add(&z, &x, &y);
//! ir.mul(&z, &z, &ten);
//!
//! ir.label("end");
//! let bytecode = ir.build();
//! # let _ = bytecode;
//! ```

use std::collections::HashMap;

use graftvm_bytecode::{Addr, Opcode, Width};
use graftvm_liternal::{Int, Liternal, UInt, Float};

/// A handle to a variable — a named slot in the current window.
#[derive(Clone, Debug)]
pub struct Var {
    pub name: String,
    pub window: usize,
    pub slot: usize,
    pub width: Width,
}

/// A handle to a label for control-flow fixup.
#[derive(Clone, Debug)]
pub struct Label {
    pub name: String,
}

/// A function descriptor.
#[derive(Clone, Debug)]
pub struct Func {
    pub name: String,
    pub window: usize,
    pub params: Vec<Var>,
    pub ret: Option<Var>,
}

/// High-level bytecode builder. Manages variables, constants, labels, and
/// window scopes so you never touch raw `Opcode` or `Addr`.
pub struct IrBuilder {
    bytecode: Vec<(Opcode, usize)>,  // (opcode, source_line for debugging)
    next_line: usize,

    // Window & slot tracking
    current_window: usize,
    window_slot_len: Vec<usize>,     // slot count per window

    // Variables
    vars: Vec<Var>,

    // Constant pool tracking
    const_slot: usize,

    // Labels for fixup
    label_defs: HashMap<String, usize>,  // label → bytecode index
    label_fixups: Vec<(String, usize)>,  // (label, bytecode index to patch)

    // Debug annotation
    annotations: Vec<(String, usize)>,   // (text, bytecode index)
}

impl IrBuilder {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            next_line: 0,
            current_window: 0,
            window_slot_len: vec![0],
            vars: Vec::new(),
            const_slot: 0,
            label_defs: HashMap::new(),
            label_fixups: Vec::new(),
            annotations: Vec::new(),
        }
    }

    // ── Variables ──

    /// Allocate a new variable in the current window.
    pub fn var(&mut self, name: &str, width: Width) -> Var {
        let slot = self.window_slot_len[self.current_window];
        self.window_slot_len[self.current_window] = slot + 1;
        let var = Var {
            name: name.to_string(),
            window: self.current_window,
            slot,
            width,
        };
        self.vars.push(var.clone());
        var
    }

    /// Return the latest variable with the given name, if any.
    pub fn find_var(&self, name: &str) -> Option<&Var> {
        self.vars.iter().rev().find(|v| v.name == name)
    }

    fn addr(&self, var: &Var) -> Addr {
        Addr { window: var.window, slot: var.slot }
    }

    // ── Constants ──

    // Insert a literal into the constant pool and return its index.
    fn pool(&mut self, lit: Liternal) -> usize {
        let idx = self.const_slot;
        self.const_slot += 1;
        self.emit_op(Opcode::StoreData { index: idx, data: lit });
        idx
    }

    /// Create a variable holding a compile-time constant.
    pub fn constant(&mut self, name: &str, lit: Liternal) -> Var {
        let idx = self.pool(lit.clone());
        let var = self.var(name, width_of_literal(&lit));
        self.emit_op(Opcode::LoadData { dst: self.addr(&var), index: idx });
        var
    }

    /// Shorthand: i32 constant.
    pub fn i32(&mut self, name: &str, v: i32) -> Var {
        self.constant(name, Liternal::Int(Int::Int32(v)))
    }

    /// Shorthand: i64 constant.
    pub fn i64(&mut self, name: &str, v: i64) -> Var {
        self.constant(name, Liternal::Int(Int::Int64(v)))
    }

    /// Shorthand: f64 constant.
    pub fn f64(&mut self, name: &str, v: f64) -> Var {
        self.constant(name, Liternal::Float(Float::Float64(v)))
    }

    /// Shorthand: bool constant.
    pub fn boolean(&mut self, name: &str, v: bool) -> Var {
        self.constant(name, Liternal::Bool(v))
    }

    /// Shorthand: u32 constant.
    pub fn u32(&mut self, name: &str, v: u32) -> Var {
        self.constant(name, Liternal::UInt(UInt::UInt32(v)))
    }

    // ── Arithmetic ──

    /// dst = lhs + rhs
    pub fn add(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Add { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: dst.width });
    }

    /// dst = lhs - rhs
    pub fn sub(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Sub { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: dst.width });
    }

    /// dst = lhs * rhs
    pub fn mul(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Mul { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: dst.width });
    }

    /// dst = lhs / rhs
    pub fn div(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Div { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: dst.width });
    }

    /// dst = lhs % rhs
    pub fn rem(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Rem { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: dst.width });
    }

    /// dst = -src
    pub fn neg(&mut self, dst: &Var, src: &Var) {
        self.emit_op(Opcode::Neg { dst: self.addr(dst), src: self.addr(src), ty: dst.width });
    }

    // ── Bitwise ──

    pub fn and(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::And { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: dst.width });
    }
    pub fn or(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Or { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: dst.width });
    }
    pub fn xor(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Xor { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: dst.width });
    }
    pub fn not(&mut self, dst: &Var, src: &Var) {
        self.emit_op(Opcode::Not { dst: self.addr(dst), src: self.addr(src), ty: dst.width });
    }

    // ── Shift ──

    pub fn shl(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Shl { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: dst.width });
    }
    pub fn shr(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Shr { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: dst.width });
    }

    // ── Comparison ──

    /// dst = lhs < rhs  (result is bool)
    pub fn lt(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Lt { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: lhs.width });
    }
    pub fn le(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Le { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: lhs.width });
    }
    pub fn gt(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Gt { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: lhs.width });
    }
    pub fn ge(&mut self, dst: &Var, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Ge { dst: self.addr(dst), lhs: self.addr(lhs), rhs: self.addr(rhs), ty: lhs.width });
    }

    /// Compare two values and set the comparison flag. Use with `branch`.
    pub fn eq(&mut self, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Eq { lhs: self.addr(lhs), rhs: self.addr(rhs) });
    }
    pub fn neq(&mut self, lhs: &Var, rhs: &Var) {
        self.emit_op(Opcode::Neq { lhs: self.addr(lhs), rhs: self.addr(rhs) });
    }

    // ── Conversion ──

    pub fn extend(&mut self, dst: &Var, src: &Var) {
        self.emit_op(Opcode::Extend { dst: self.addr(dst), src: self.addr(src), signed: src.width.is_int() });
    }
    pub fn trunc(&mut self, dst: &Var, src: &Var) {
        self.emit_op(Opcode::Trunc { dst: self.addr(dst), src: self.addr(src) });
    }
    pub fn reinterpret(&mut self, dst: &Var, src: &Var) {
        self.emit_op(Opcode::Reinterpret { dst: self.addr(dst), src: self.addr(src) });
    }
    pub fn convert(&mut self, dst: &Var, src: &Var) {
        self.emit_op(Opcode::Convert { dst: self.addr(dst), src: self.addr(src), signed: src.width.is_int() });
    }

    // ── Data movement ──

    /// Move ownership from src to dst (src becomes empty).
    pub fn move_var(&mut self, dst: &Var, src: &Var) {
        self.emit_op(Opcode::Move { dst: self.addr(dst), src: self.addr(src) });
    }

    /// Copy value from src to dst.
    pub fn copy_var(&mut self, dst: &Var, src: &Var) {
        self.emit_op(Opcode::Copy { dst: self.addr(dst), src: self.addr(src) });
    }

    /// Drop (deallocate) a variable.
    pub fn drop_var(&mut self, var: &Var) {
        self.emit_op(Opcode::Drop { src: self.addr(var) });
    }

    // ── Argument passing ──

    /// Push a value from `src` onto the argument stack.
    pub fn push_arg(&mut self, src: &Var) {
        self.emit_op(Opcode::PushArg { src: self.addr(src) });
    }

    /// Pop a value from the argument stack into `dst`.
    pub fn pop_arg(&mut self, dst: &Var) {
        self.emit_op(Opcode::PopArg { dst: self.addr(dst) });
    }

    // ── Control flow ──

    /// Create a named label.
    pub fn label(&mut self, name: &str) -> Label {
        let label = Label { name: name.to_string() };
        let idx = self.bytecode.len();
        self.label_defs.insert(name.to_string(), idx);
        // Patch any pending fixups for this label
        let mut i = 0;
        while i < self.label_fixups.len() {
            let (lbl, pos) = &self.label_fixups[i];
            if lbl == name {
                // Replace the placeholder opcode with the real address
                let (op, _) = &self.bytecode[*pos];
                let patched = match *op {
                    Opcode::Jump(_) => Opcode::Jump(idx),
                    Opcode::Branch(_) => Opcode::Branch(idx),
                    Opcode::Call(_) => Opcode::Call(idx),
                    _ => unreachable!(),
                };
                self.bytecode[*pos].0 = patched;
                self.label_fixups.swap_remove(i);
            } else {
                i += 1;
            }
        }
        label
    }

    /// Unconditional jump to a label.
    pub fn jump(&mut self, label: &str) {
        let pos = self.bytecode.len();
        if let Some(&target) = self.label_defs.get(label) {
            self.emit_op(Opcode::Jump(target));
        } else {
            // Forward reference — emit placeholder and record fixup
            self.emit_op(Opcode::Jump(0));
            self.label_fixups.push((label.to_string(), pos));
        }
    }

    /// Conditional branch: jump to label if the comparison flag is set.
    pub fn branch(&mut self, label: &str) {
        let pos = self.bytecode.len();
        if let Some(&target) = self.label_defs.get(label) {
            self.emit_op(Opcode::Branch(target));
        } else {
            self.emit_op(Opcode::Branch(0));
            self.label_fixups.push((label.to_string(), pos));
        }
    }

    /// Call a named function label.
    /// The called function must end with `Exit` + `Ret`, which will pop the
    /// child window and return here. This method tracks the window pop in the
    /// compiler to keep `current_window` in sync with runtime.
    pub fn call(&mut self, label: &str) {
        let pos = self.bytecode.len();
        if let Some(&target) = self.label_defs.get(label) {
            self.emit_op(Opcode::Call(target));
        } else {
            self.emit_op(Opcode::Call(0));
            self.label_fixups.push((label.to_string(), pos));
        }
        // The called function will Exit, popping one runtime window.
        self.current_window = self.current_window.saturating_sub(1);
    }

    /// Return from a function.
    pub fn ret(&mut self) {
        self.emit_op(Opcode::Ret);
    }

    // ── Scopes / Functions ──

    /// Enter a new window scope (e.g. function body).
    /// Resets the slot counter for this window level so that fresh variable
    /// allocation starts from slot 0.
    pub fn enter(&mut self) {
        self.current_window += 1;
        if self.current_window >= self.window_slot_len.len() {
            self.window_slot_len.push(0);
        } else {
            // Reset slot counter — this window level may have been used by a
            // previous scope (e.g. a function body compiled with enter_silent).
            self.window_slot_len[self.current_window] = 0;
        }
        self.emit_op(Opcode::Enter);
    }

    /// Enter a new window scope for tracking purposes only (no `Enter` opcode
    /// emitted — the caller is responsible for emitting it).
    pub fn enter_silent(&mut self) {
        self.current_window += 1;
        if self.current_window >= self.window_slot_len.len() {
            self.window_slot_len.push(0);
        }
    }

    /// Exit the current window scope.
    pub fn exit(&mut self) {
        self.emit_op(Opcode::Exit);
        self.current_window = self.current_window.saturating_sub(1);
    }

    /// Exit tracking without emitting the `Exit` opcode.
    pub fn exit_silent(&mut self) {
        self.current_window = self.current_window.saturating_sub(1);
    }

    /// Define a function that allocates its own window, with named parameters.
    pub fn func(&mut self, name: &str, params: &[(&str, Width)], ret: Option<Width>) -> Func {
        let window = self.current_window + 1;
        let func = Func {
            name: name.to_string(),
            window,
            params: Vec::new(),
            ret: None,
        };

        self.enter();

        // Allocate parameter vars in the new window
        // (The caller is expected to Move values into these slots before calling)
        let mut func = func;
        for (pname, pwidth) in params {
            let v = self.var(pname, *pwidth);
            func.params.push(v);
        }

        if let Some(w) = ret {
            func.ret = Some(self.var(&format!("{}.ret", name), w));
        }

        self.annotations.push((format!("func {}", name), self.bytecode.len()));
        func
    }

    /// End a function: emit Exit.
    pub fn end_func(&mut self, _func: &Func) {
        self.exit();
    }

    // ── System ──

    /// Perform a system call: `n=0` writes `src` value to stdout, result in `dst`.
    pub fn syscall(&mut self, n: u32, src: &Var, dst: &Var) {
        self.emit_op(Opcode::Syscall {
            n,
            src: self.addr(src),
            dst: self.addr(dst),
        });
    }

    /// Call a native function by name, popping `arity` args from the arg stack.
    pub fn call_native(&mut self, name: &str, arity: u32) {
        self.emit_op(Opcode::CallNative {
            name: name.to_string(),
            arity,
        });
    }

    // ── Build ──

    /// Consume the builder and produce the final bytecode.
    pub fn build(self) -> Vec<Opcode> {
        if !self.label_fixups.is_empty() {
            let unresolved: Vec<_> = self.label_fixups.iter().map(|(l, _)| l.clone()).collect();
            panic!("unresolved label fixups: {:?}", unresolved);
        }
        self.bytecode.into_iter().map(|(op, _)| op).collect()
    }

    /// Build and also return debug annotations (label, function boundaries).
    pub fn build_with_annotations(self) -> (Vec<Opcode>, Vec<(String, usize)>) {
        if !self.label_fixups.is_empty() {
            let unresolved: Vec<_> = self.label_fixups.iter().map(|(l, _)| l.clone()).collect();
            panic!("unresolved label fixups: {:?}", unresolved);
        }
        let bytecode = self.bytecode.into_iter().map(|(op, _)| op).collect();
        (bytecode, self.annotations)
    }

    fn emit_op(&mut self, op: Opcode) {
        let line = self.next_line;
        self.next_line += 1;
        self.bytecode.push((op, line));
    }
}

fn width_of_literal(lit: &Liternal) -> Width {
    match lit {
        Liternal::Int(Int::Int8(_)) => Width::I8,
        Liternal::Int(Int::Int16(_)) => Width::I16,
        Liternal::Int(Int::Int32(_)) => Width::I32,
        Liternal::Int(Int::Int64(_)) => Width::I64,
        Liternal::UInt(UInt::UInt8(_)) => Width::U8,
        Liternal::UInt(UInt::UInt16(_)) => Width::U16,
        Liternal::UInt(UInt::UInt32(_)) => Width::U32,
        Liternal::UInt(UInt::UInt64(_)) => Width::U64,
        Liternal::Float(Float::Float32(_)) => Width::F32,
        Liternal::Float(Float::Float64(_)) => Width::F64,
        Liternal::Bool(_) => Width::I8,       // bool stored as i8
        Liternal::String(_) => Width::I64,     // string as pointer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_add() {
        let mut ir = IrBuilder::new();
        let x = ir.i32("x", 3);
        let y = ir.i32("y", 7);
        let z = ir.var("z", Width::I32);
        ir.add(&z, &x, &y);
        let bc = ir.build();
        assert_eq!(bc.len(), 5);
    }

    #[test]
    fn smoke_func() {
        let mut ir = IrBuilder::new();
        let f = ir.func("add", &[("a", Width::I32), ("b", Width::I32)], Some(Width::I32));
        ir.add(f.ret.as_ref().unwrap(), &f.params[0], &f.params[1]);
        ir.end_func(&f);
        let bc = ir.build();
        assert!(bc.len() >= 3);
    }

    #[test]
    fn label_forward_ref() {
        let mut ir = IrBuilder::new();
        let x = ir.i32("x", 42);
        let y = ir.i32("y", 0);
        ir.eq(&x, &y);
        ir.branch("skip");
        let z = ir.var("z", Width::I32);
        ir.copy_var(&z, &x);
        ir.label("skip");
        let bc = ir.build();
        assert_eq!(bc.len(), 7);
    }
}
