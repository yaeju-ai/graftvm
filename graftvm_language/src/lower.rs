use std::collections::HashMap;

use graftvm_bytecode::{Opcode, Width};
use graftvm_ir::{IrBuilder, Var};
use crate::parser::{Expr, ParseResult};

/// Compile parsed Parlance source into GraftVM bytecode.
pub fn lower(result: ParseResult) -> Result<Vec<Opcode>, String> {
    let mut ctx = Context::new(
        result.infixes.iter().map(|i| (i.symbol.clone(), i.precedence)).collect(),
    );

    // Register function names so recursive refs work.
    for func_def in &result.funcs {
        ctx.funcs.insert(func_def.name.clone());
    }

    // Register a dummy Var for each function name (so Var refs in the body work).
    let mut ir = IrBuilder::new();
    for func_def in &result.funcs {
        let _p = ir.var(&func_def.name, Width::I64);
        ctx.scope.insert(func_def.name.clone(), _p);
    }

    // ── Compile function definitions ──
    // Each function body is compiled under a label. A Jump over the body
    // prevents inline execution. The body ends with PushArg (return value),
    // Exit (pop the runtime window), and Ret (jump back to caller).
    for func_def in &result.funcs {
        let skip_label = format!("{}_skip", func_def.name);
        ir.jump(&skip_label);

        // Function entry label
        ir.label(&func_def.name);

        // Enter silently — no Enter opcode emitted (the caller's Enter
        // creates the runtime window). We track it in the compiler so
        // that ir.var() allocates slots in the right window.
        ir.enter_silent();

        // Register parameters as variables (already in slots 0..N-1,
        // placed there by the caller before the Call).
        for param in &func_def.params {
            let v = ir.var(param, Width::I64);
            ctx.scope.insert(param.clone(), v);
        }

        // Compile body
        let body_var = lower_expr(&mut ir, &mut ctx, &func_def.body)?;

        // Move return value to a dedicated ret var
        let ret_var = ir.var(&format!("{}.ret", func_def.name), Width::I64);
        ir.copy_var(&ret_var, &body_var);

        // Push return value onto arg stack, emit Exit (pop runtime window),
        // then Ret (return to caller).
        ir.push_arg(&ret_var);
        ir.exit();       // emits Exit opcode – pops the runtime window
        ir.ret();        // emits Ret opcode – pops return address, jumps back

        // Clean up scope
        for param in &func_def.params {
            ctx.scope.remove(param);
        }

        ir.label(&skip_label);
    }

    // ── Compile top-level expressions ──
    for expr in &result.exprs {
        lower_expr(&mut ir, &mut ctx, expr)?;
    }

    Ok(ir.build())
}

struct Context {
    scope: HashMap<String, Var>,
    #[allow(dead_code)]
    infixes: HashMap<String, u32>,
    /// Set of function names known to the compiler.
    funcs: std::collections::HashSet<String>,
}

impl Context {
    fn new(infixes: HashMap<String, u32>) -> Self {
        Self { scope: HashMap::new(), infixes, funcs: std::collections::HashSet::new() }
    }

    fn lookup(&self, name: &str) -> Option<&Var> {
        self.scope.get(name)
    }
}

fn lower_expr(ir: &mut IrBuilder, ctx: &mut Context, expr: &Expr) -> Result<Var, String> {
    match expr {
        Expr::Int(n) => Ok(ir.i64(&format!("lit_{}", n), *n)),
        Expr::Float(n) => Ok(ir.f64(&format!("lit_{}", n), *n)),
        Expr::Bool(b) => Ok(ir.boolean(&format!("lit_{}", b), *b)),
        Expr::String(s) => Ok(ir.i64(&format!("str_{}", s.len()), s.len() as i64)),
        Expr::Var(name) => ctx.lookup(name).cloned().ok_or_else(|| format!("undefined variable: {}", name)),

        Expr::Apply { func, args } => {
            if let Expr::Var(op) = func.as_ref() {
                match op.as_str() {
                    "+" | "-" | "*" | "/" | "%" => return lower_binop(ir, ctx, op, args),
                    "<" | "<=" | ">" | ">=" => return lower_compare(ir, ctx, op, args),
                    "==" => return lower_eq(ir, ctx, args, true),
                    "!=" => return lower_eq(ir, ctx, args, false),
                    _ if ctx.funcs.contains(op.as_str()) => {
                        return lower_func_call(ir, ctx, op, args)
                    }
                    _ => {} // unknown — fall through
                }
            }
            // Generic fallback: lower args, return dummy
            let _f = lower_expr(ir, ctx, func)?;
            for arg in args { lower_expr(ir, ctx, arg)?; }
            Ok(ir.var("apply", Width::I64))
        }

        Expr::Lambda { params, body } => {
            ir.enter();
            for param in params.iter() {
                let v = ir.var(param.as_str(), Width::I64);
                ctx.scope.insert(param.clone(), v);
            }
            let result = lower_expr(ir, ctx, body)?;
            let outer = ir.var("lam_res", Width::I64);
            ir.copy_var(&outer, &result);
            ir.exit();
            Ok(outer)
        }

        Expr::Let { bindings, body } => {
            for (name, val_expr) in bindings {
                let val = lower_expr(ir, ctx, val_expr)?;
                let v = ir.var(name, Width::I64);
                ir.copy_var(&v, &val);
                ctx.scope.insert(name.to_string(), v);
            }
            lower_expr(ir, ctx, body)
        }

        Expr::If { cond, then, else_ } => {
            let cond_var = lower_expr(ir, ctx, cond)?;
            let zero = ir.i64("zero", 0);
            let result = ir.var("if_res", Width::I64);
            // cond == 0 → jump to else; otherwise fall through to then
            ir.eq(&cond_var, &zero);
            ir.branch("if_else");
            // then branch (fallthrough)
            let then_var = lower_expr(ir, ctx, then)?;
            ir.copy_var(&result, &then_var);
            ir.jump("if_done");
            // else branch
            ir.label("if_else");
            let else_var = lower_expr(ir, ctx, else_)?;
            ir.copy_var(&result, &else_var);
            ir.label("if_done");
            Ok(result)
        }
    }
}

/// Lower a user-defined function call:
///   PushArg(arg0), PushArg(arg1), ...
///   Enter                          (create callee window)
///   PopArg(param_N-1), ..., PopArg(param_0)   (pop into callee slots)
///   Call func_label                (push ret_addr, jump; callee does Exit+Ret)
///   PopArg(result)                 (pop return value into caller's window)
fn lower_func_call(ir: &mut IrBuilder, ctx: &mut Context, name: &str, args: &[Expr]) -> Result<Var, String> {
    // Push args onto the arg stack (evaluated in the caller's window)
    for arg in args {
        let v = lower_expr(ir, ctx, arg)?;
        ir.push_arg(&v);
    }

    // Enter the callee's window
    ir.enter();

    // Pop args into callee's slots (reverse order — stack is LIFO)
    for i in (0..args.len()).rev() {
        let param = ir.var(&format!("{}_{}", name, i), Width::I64);
        ir.pop_arg(&param);
    }

    // Allocate a result variable in the *caller's* window.
    // ir.call() decrements current_window to account for the callee's Exit,
    // so the result variable ends up in the caller's window.
    let result = ir.var(&format!("{}_call", name), Width::I64);

    // Call the function (decrements current_window by 1)
    ir.call(name);

    // Pop return value into result variable (current window is caller's now)
    ir.pop_arg(&result);

    Ok(result)
}

fn lower_binop(ir: &mut IrBuilder, ctx: &mut Context, op: &str, args: &[Expr]) -> Result<Var, String> {
    if args.len() != 2 { return Err(format!("{} expects 2 args", op)); }
    let lhs = lower_expr(ir, ctx, &args[0])?;
    // Save LHS to a temp slot so it isn't clobbered by a function call in RHS
    let lhs_saved = ir.var(&format!("{}_saved", op), Width::I64);
    ir.copy_var(&lhs_saved, &lhs);
    let rhs = lower_expr(ir, ctx, &args[1])?;
    let r = ir.var(&format!("{}_r", op), Width::I64);
    match op {
        "+" => ir.add(&r, &lhs_saved, &rhs),
        "-" => ir.sub(&r, &lhs_saved, &rhs),
        "*" => ir.mul(&r, &lhs_saved, &rhs),
        "/" => ir.div(&r, &lhs_saved, &rhs),
        "%" => ir.rem(&r, &lhs_saved, &rhs),
        _ => unreachable!(),
    }
    Ok(r)
}

fn lower_compare(ir: &mut IrBuilder, ctx: &mut Context, op: &str, args: &[Expr]) -> Result<Var, String> {
    if args.len() != 2 { return Err(format!("{} expects 2 args", op)); }
    let lhs = lower_expr(ir, ctx, &args[0])?;
    let lhs_saved = ir.var(&format!("{}_saved", op), Width::I64);
    ir.copy_var(&lhs_saved, &lhs);
    let rhs = lower_expr(ir, ctx, &args[1])?;
    let r = ir.var(&format!("{}_r", op), Width::I8);
    match op {
        "<" => ir.lt(&r, &lhs_saved, &rhs),
        "<=" => ir.le(&r, &lhs_saved, &rhs),
        ">" => ir.gt(&r, &lhs_saved, &rhs),
        ">=" => ir.ge(&r, &lhs_saved, &rhs),
        _ => unreachable!(),
    }
    Ok(r)
}

fn lower_eq(ir: &mut IrBuilder, ctx: &mut Context, args: &[Expr], eq: bool) -> Result<Var, String> {
    if args.len() != 2 { return Err(format!("== expects 2 args")); }
    let lhs = lower_expr(ir, ctx, &args[0])?;
    let lhs_saved = ir.var(&format!("eq_saved"), Width::I64);
    ir.copy_var(&lhs_saved, &lhs);
    let rhs = lower_expr(ir, ctx, &args[1])?;
    if eq { ir.eq(&lhs_saved, &rhs); } else { ir.neq(&lhs_saved, &rhs); }
    let tmp = ir.var("cmp_r", Width::I8);
    let one = ir.i64("one", 1);
    let zero = ir.i64("zero", 0);
    ir.copy_var(&tmp, &one);
    ir.branch("cmp_d");
    ir.copy_var(&tmp, &zero);
    ir.label("cmp_d");
    Ok(tmp)
}
