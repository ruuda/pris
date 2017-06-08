grammar Pris;

// options { k = 1; }

tokens { IDENT, COLOR, NUM, STRING_LITERAL, RAW_STRING_LITERAL }

statement
  : import_
  | assign
  | return_
  | block
  | put_at
  ;

import_: 'import' idents;

idents: IDENT | idents '.' IDENT;

assign: IDENT '=' expr;

expr: expr_add;

expr_add
  : expr_mul
  | expr_add '+' expr_mul
  | expr_add '-' expr_mul
  | expr_add '~' expr_mul
  ;

expr_mul
  : expr_exp
  | expr_mul '*' expr_exp
  | expr_mul '/' expr_exp
  ;

expr_exp
  : term
  | term '^' term
  | '-' term
  | fn_call
  ;

term
  : string
  | num
  | COLOR
  | idents
  | coord
  | fn_def
  | block
  | '(' expr ')'
  ;

num: NUM ('w' | 'h' | 'em' | 'pt')?;

string: STRING_LITERAL | RAW_STRING_LITERAL;

coord: '(' expr ',' expr ')';

fn_call
  : term '(' ')'
  | term '(' fn_call_args ')'
  ;

fn_call_args: expr | fn_call_args ',' expr;

fn_def
  : 'function' '(' ')'
  | 'function' '(' fn_def_args ')'
  ;

fn_def_args: IDENT | fn_def_args ',' IDENT;

block: '{' statement* '}';

return_: 'return' expr;

put_at
  : 'put' expr 'at' expr
  | 'at' expr 'put' expr
  ;
