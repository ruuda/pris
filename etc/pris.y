%{
// This file contains a Bison grammar for Pris. It is not used; the actual
// parser is a hand-written parser in src/parser.rs. We still keep this file
// around because it is a nice and readable -- to humans and machines --
// specification of the grammar. In particular, Bison will warn about
// ambiguities that might sneak in.
%}

%token IDENT COLOR NUMBER STRING

%%

statements: statement | statements statement;

statement
  : import
  | assign
  | return
  | block
  | put
  ;

import: "import" idents;

idents: IDENT | idents '.' IDENT;

assign: IDENT '=' expr;

expr: expr_at | expr_add;

expr_at: expr_add "at" expr_at;

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
  : STRING
  | number
  | COLOR
  | idents
  | coord
  | fn_def
  | block
  | '(' expr ')'
  ;

suffix: "w" | "h" | "em" | "pt";

number
  : NUMBER
  | NUMBER suffix;

coord: '(' expr ',' expr ')';

fn_call
  : term '(' ')'
  | term '(' fn_call_args ')'
  ;

fn_call_args: expr | fn_call_args ',' expr;

fn_def
  : "function" '(' ')'
  | "function" '(' fn_def_args ')'
  ;

fn_def_args: IDENT | fn_def_args ',' IDENT;

block
  : '{' '}'
  | '{' statements '}'
  ;

return: "return" expr;

put: "put" expr;
