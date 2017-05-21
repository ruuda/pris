// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::rc::Rc;

use ast;
use ast::{Assign, BinOp, BinTerm, Block, Coord, FnCall, FnDef, Idents};
use ast::{Num, PutAt, Return, Stmt, Term, UnOp, UnTerm, Unit};
use error::{Error, Result};
use elements::{Color, Vec2};
use pretty::Formatter;
use runtime::{Builtin, FontMap, Frame, Env, Subframe, Val};
use types::ValType;

// Expression interpreter.

// TODO: This should not be public at all.
pub struct ExprInterpreter<'i, 'a: 'i> {
    pub font_map: &'i mut FontMap,
    pub env: &'i Env<'a>,
}

impl<'i, 'a> ExprInterpreter<'i, 'a> {

    fn eval_expr(&mut self, term: &'a Term<'a>) -> Result<Val<'a>> {
        match *term {
            Term::String(ref s) => Ok(Val::Str(s.clone())),
            Term::Number(ref x) => self.eval_num(x),
            Term::Color(ref co) => Ok(ExprInterpreter::eval_color(co)),
            Term::Idents(ref i) => self.env.lookup(i),
            Term::Coord(ref co) => self.eval_coord(co),
            Term::BinOp(ref bo) => self.eval_binop(bo),
            Term::UnOp(ref uop) => self.eval_unop(uop),
            Term::FnCall(ref f) => self.eval_call(f),
            Term::FnDef(ref fd) => Ok(Val::FnExtrin(fd)),
            Term::Block(ref bk) => self.eval_block(bk),
        }
    }

    fn eval_num(&mut self, num: &'a Num) -> Result<Val<'a>> {
        let Num(x, opt_unit) = *num;
        if let Some(unit) = opt_unit {
            match unit {
                Unit::W => Ok(Val::Num(1920.0 * x, 1)),
                Unit::H => Ok(Val::Num(1080.0 * x, 1)),
                Unit::Pt => Ok(Val::Num(1.0 * x, 1)),
                Unit::Em => {
                    // The variable "font_size" should always be set, it is present
                    // in the global environment.
                    let ident_font_size = Idents(vec!["font_size"]);
                    let emsize = self.env.lookup_len(&ident_font_size)?;
                    Ok(Val::Num(emsize * x, 1))
                }
            }
        } else {
            Ok(Val::Num(x, 0))
        }
    }

    fn eval_color(col: &ast::Color) -> Val<'a> {
        let ast::Color(rbyte, gbyte, bbyte) = *col;
        let cf64 = Color::new(rbyte as f64 / 255.0, gbyte as f64 / 255.0, bbyte as f64 / 255.0);
        Val::Col(cf64)
    }

    fn eval_coord(&mut self, coord: &'a Coord<'a>) -> Result<Val<'a>> {
        let x = self.eval_expr(&coord.0)?;
        let y = self.eval_expr(&coord.1)?;
        match (x, y) {
            (Val::Num(a, d), Val::Num(b, e)) if d == e => Ok(Val::Coord(a, b, d)),
            _ => {
                let msg = "Type error: coord must be (num, num) or (len, len), \
                           but found (<TODO>, <TODO>) instead.";
                Err(Error::Other(String::from(msg)))
            }
        }
    }

    fn eval_binop(&mut self, binop: &'a BinTerm<'a>) -> Result<Val<'a>> {
        let lhs = self.eval_expr(&binop.0)?;
        let rhs = self.eval_expr(&binop.2)?;
        match binop.1 {
            BinOp::Adj => ExprInterpreter::eval_adj(lhs, rhs),
            BinOp::Add => ExprInterpreter::eval_add(lhs, rhs),
            BinOp::Sub => ExprInterpreter::eval_sub(lhs, rhs),
            BinOp::Mul => ExprInterpreter::eval_mul(lhs, rhs),
            BinOp::Div => ExprInterpreter::eval_div(lhs, rhs),
            BinOp::Exp => panic!("TODO: eval exp"),
        }
    }

    /// Adjoins two frames.
    fn eval_adj(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
        match (lhs, rhs) {
            (Val::Frame(f0), Val::Frame(f1)) => {
                let mut frame = (*f0).clone();
                let anchor = f0.get_anchor();
                // Copy the elements of f1 onto the new frame (cloned from f0),
                // subframe by subframe.
                for (i, sf1) in f1.get_subframes().iter().enumerate() {
                    // If f1 had more subframes than f0, the result will have as
                    // many subframes as f1.
                    if frame.get_subframes().len() <= i {
                        frame.push_subframe(Subframe::new());
                    }
                    let mut subframe = frame.get_subframe_mut(i);
                    for pe in sf1.get_elements() {
                        subframe.place_element(anchor + pe.position, pe.element.clone());
                    }
                }
                frame.set_anchor(anchor + f1.get_anchor());
                frame.union_bounding_box(&f1.get_bounding_box().offset(anchor));
                Ok(Val::Frame(Rc::new(frame)))
            }
            (lhs, rhs) => {
                Err(Error::binop_type("~", ValType::Frame, lhs.get_type(), rhs.get_type()))
            }
        }
    }

    fn eval_add(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
        match (lhs, rhs) {
            (Val::Num(x0, d0), Val::Num(x1, d1)) if d0 == d1 => {
                Ok(Val::Num(x0 + x1, d0))
            }
            (Val::Coord(x0, y0, d0), Val::Coord(x1, y1, d1)) if d0 == d1 => {
                Ok(Val::Coord(x0 + x1, y0 + y1, d0))
            }
            (Val::Str(a), Val::Str(b)) => {
                Ok(Val::Str(a + &b))
            }
            (lhs, rhs) => {
                let mut f = Formatter::new();
                f.print("Type error: '+' expects operands of the same type, \
                         num or len or coords thereof, \
                         but found '");
                f.print(lhs);
                f.print("' and '");
                f.print(rhs);
                f.print("' instead.");
                Err(Error::Other(f.into_string()))
            }
        }
    }

    fn eval_sub(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
        match (lhs, rhs) {
            (Val::Num(x0, d0), Val::Num(x1, d1)) if d0 == d1 => {
                Ok(Val::Num(x0 - x1, d0))
            }
            (Val::Coord(x0, y0, d0), Val::Coord(x1, y1, d1)) if d0 == d1 => {
                Ok(Val::Coord(x0 - x1, y0 - y1, d0))
            }
            (lhs, rhs) => {
                let mut f = Formatter::new();
                f.print("Type error: '-' expects operands of the same type, \
                         num or len or coords thereof, \
                         but found '");
                f.print(lhs);
                f.print("' and '");
                f.print(rhs);
                f.print("' instead.");
                Err(Error::Other(f.into_string()))
            }
        }
    }

    fn eval_mul(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
        match (lhs, rhs) {
            (Val::Num(x, d), Val::Num(y, e)) => Ok(Val::Num(x * y, d + e)),
            (Val::Coord(x, y, d), Val::Num(z, e)) => Ok(Val::Coord(x * z, y * z, d + e)),
            (Val::Num(z, e), Val::Coord(x, y, d)) => Ok(Val::Coord(x * z, y * z, d + e)),
            _ => {
                let msg = "Type error: '*' expects num or len operands, \
                           but found <TODO> and <TODO> instead.";
                Err(Error::Other(String::from(msg)))
            }
        }
    }

    fn eval_div(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
        match (lhs, rhs) {
            (Val::Num(x, d), Val::Num(y, e)) => Ok(Val::Num(x / y, d - e)),
            _ => {
                let msg = "Type error: '/' expects num or len operands, \
                           but found <TODO> and <TODO> instead.";
                Err(Error::Other(String::from(msg)))
            }
        }
    }

    fn eval_unop(&mut self, unop: &'a UnTerm<'a>) -> Result<Val<'a>> {
        let rhs = self.eval_expr(&unop.1)?;
        match unop.0 {
            UnOp::Neg => ExprInterpreter::eval_neg(rhs),
        }
    }

    fn eval_neg(rhs: Val<'a>) -> Result<Val<'a>> {
        match rhs {
            Val::Num(x, d) => Ok(Val::Num(-x, d)),
            Val::Coord(x, y, d) => Ok(Val::Coord(-x, -y, d)),
            _ => {
                let msg = "Type error: '-' expects a num or len operand, \
                           but found <TODO> instead.";
                Err(Error::Other(String::from(msg)))
            }
        }
    }

    fn eval_call(&mut self, call: &'a FnCall<'a>) -> Result<Val<'a>> {
        let mut args = Vec::with_capacity(call.1.len());
        for arg in &call.1 {
            args.push(self.eval_expr(arg)?);
        }
        let func = self.eval_expr(&call.0)?;
        match func {
            // For a user-defined function, we evaluate the function body.
            Val::FnExtrin(fn_def) => self.eval_call_extrin(fn_def, args),
            // For a builtin function, the value carries a function pointer,
            // so we can just call that.
            Val::FnIntrin(Builtin(intrin)) => intrin(self, args),
            // Other things are not callable.
            _ => {
                let msg = "Type error: attempting to call value of type <TODO>. \
                           Only functions can be called.";
                Err(Error::Other(String::from(msg)))
            }
        }
    }

    fn eval_call_extrin(&mut self,
                        fn_def: &'a FnDef<'a>,
                        args: Vec<Val<'a>>)
                        -> Result<Val<'a>> {
        // Ensure that a value is provided for every argument, and no more.
        if fn_def.0.len() != args.len() {
            let msg = format!("Arity error: function takes {} arguments, \
                               but {} were provided.",
                              fn_def.0.len(), args.len());
            return Err(Error::Other(msg))
        }

        // For a function call, bring the argument in scope as variables, and
        // then evaluate the body block in the modified environment.
        let mut inner_env = self.env.clone();
        for (arg_name, val) in fn_def.0.iter().zip(args) {
            inner_env.put(arg_name, val);
        }

        let mut inner_interpreter = ExprInterpreter {
            font_map: &mut *self.font_map,
            env: &inner_env,
        };

        inner_interpreter.eval_block(&fn_def.1)
    }

    fn eval_block(&mut self, block: &'a Block<'a>) -> Result<Val<'a>> {
        // A block is evaluated in its enclosing environment, but it does not
        // modify the environment, it gets a copy.
        let inner_env = (*self.env).clone();

        // A block can consist of multiple statements, which mutate the frame,
        // until the block ends. The statement interpreter keeps track of this
        // frame internally. When the block ends, the frame is the result of the
        // block (if there was no return).
        let mut stmt_interpreter = StmtInterpreter {
            font_map: self.font_map,
            frame: Frame::from_env(inner_env),
            current_subframe: 0,
        };

        for statement in &block.0 {
            match *statement {
                // A return statement in a block determines the value that the
                // block evalates to, if a return is present.
                Stmt::Return(Return(ref r)) => {
                    return stmt_interpreter.get_expr_interpreter().eval_expr(r)
                }
                // A block statemen to make a frame can only be used at the top
                // level.
                Stmt::Block(..) => {
                    let msg = "Error: slides can only be introduced at the top level. \
                               Note: use 'at (0w, 0w) put { ... }' to place a frame.";
                    return Err(Error::Other(String::from(msg)));
                }
                // Otherwise, evaluating a statement only mutates the frame.
                _ => {
                    let maybe_frame = stmt_interpreter.eval_statement(statement)?;
                    assert!(maybe_frame.is_none());
                }
            }
        }

        // The result of a block, if there was no return, is the frame in its
        // final state.
        Ok(Val::Frame(Rc::new(stmt_interpreter.frame)))
    }
}

// Statement interpreter.

// TODO: This should not be public, or at least, not in this form.
pub struct StmtInterpreter<'i, 'a: 'i> {
    font_map: &'i mut FontMap,
    frame: Frame<'a>,
    current_subframe: usize,
}

impl<'i, 'a> StmtInterpreter<'i, 'a> {

    pub fn new(font_map: &'i mut FontMap) -> StmtInterpreter<'i, 'a> {
        StmtInterpreter {
            font_map: font_map,
            frame: Frame::new(),
            current_subframe: 0,
        }
    }

    fn get_expr_interpreter<'j>(&'j mut self) -> ExprInterpreter<'j, 'a> {
        let env = self.frame.get_env();
        ExprInterpreter {
            font_map: self.font_map,
            env: env,
        }
    }

    pub fn eval_statement(&mut self,
                          stmt: &'a Stmt<'a>)
                          -> Result<Option<Rc<Frame<'a>>>> {
        match *stmt {
            Stmt::Import(ref _i) => {
                println!("TODO: eval import");
                Ok(None)
            }
            Stmt::Assign(ref a) => {
                self.eval_assign(a)?;
                Ok(None)
            }
            Stmt::Return(..) => {
                // The return case is handled in block evaluation. A bare return
                // statement does not make sense.
                let msg = "Syntax error: 'return' cannot be used here.";
                Err(Error::Other(String::from(msg)))
            }
            Stmt::Block(ref bk) => {
                let mut expr_interpreter = self.get_expr_interpreter();
                if let Val::Frame(frame) = expr_interpreter.eval_block(bk)? {
                    Ok(Some(frame))
                } else {
                    let msg = "Type error: top-level blocks must evaluate to \
                               frames, but a <TODO> was encountered instead.";
                    Err(Error::Other(String::from(msg)))
                }
            }
            Stmt::PutAt(ref pa) => {
                self.eval_put_at(pa)?;
                Ok(None)
            }
        }
    }

    fn eval_assign(&mut self, stmt: &'a Assign<'a>) -> Result<()> {
        let Assign(target, ref expression) = *stmt;
        let value = self.get_expr_interpreter().eval_expr(expression)?;
        self.frame.put_in_env(target, value);
        Ok(())
    }

    fn eval_put_at(&mut self, put_at: &'a PutAt<'a>) -> Result<()> {
        let content = match self.get_expr_interpreter().eval_expr(&put_at.0)? {
            Val::Frame(f) => f,
            _ => {
                let msg = "Cannot place <TODO>. Only frames can be placed.";
                return Err(Error::Other(String::from(msg)));
            }
        };

        let pos = match self.get_expr_interpreter().eval_expr(&put_at.1)? {
            // TODO: Make Coord type carry Vec2 instead of separate x, y.
            Val::Coord(x, y, 1) => Vec2::new(x, y),
            _ => {
                let msg = "Placement requires a coordinate with length units, \
                           but a <TODO> was found instead.";
                return Err(Error::Other(String::from(msg)));
            }
        };

        // Ensure that the current frame has enough subframes to place the
        // elements in content subframes. If the content has more subframes than
        // the current frame, more must be added.
        while content.get_subframes().len() > self.frame.get_subframes().len() {
            self.frame.push_subframe(Subframe::new());
        }

        for (i, subframe) in content.get_subframes().iter().enumerate() {
            // Getting the subframe will not fail here, because we ensured that
            // they all exist before entering the loop.
            let sf_idx = self.current_subframe + i;
            let dest_sf = self.frame.get_subframe_mut(sf_idx);

            for pe in subframe.get_elements() {
                dest_sf.place_element(pos + pe.position, pe.element.clone());
            }
        }

        self.frame.union_bounding_box(&content.get_bounding_box().offset(pos));

        // Update the anchor of the frame: the anchor of a block is the anchor
        // of the element that was placed last.
        self.frame.set_anchor(pos + content.get_anchor());

        Ok(())
    }
}
