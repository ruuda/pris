%{
// This file contains a Bison grammar for Pris. It is not used; the actual
// parser is a hand-written parser in src/parser.rs. We still keep this file
// around because it is a nice and readable -- to humans and machines --
// specification of the grammar. In particular, Bison will warn about
// ambiguities that might sneak in.
//
// The grammar currently requires two tokens of lookahed in one case (see also
// the comment below), and only one token for most other cases.
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

expr: expr_infix;

expr_infix
  : expr_add
  /* Note that the presence of IDENT here causes a shift/reduce conflict; it
   * makes the grammar ambiguous in the case of a parser with one token
   * lookahead, because the IDENT here could be either an infix call, or it
   * could be the left-hand side of an assignment. But by looking ahead two
   * tokens we can tell them apart: if the IDENT is followed by '=' then it is
   * part of an assignment, otherwise it is an infix call. An other way to
   * resolve the ambiguity, without resorting to an extra token of lookahead,
   * would have been to terminate statements with semicolons.
   */
  | expr_infix IDENT expr_add;

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
  | list
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
  | term '(' fn_call_args ',' ')'
  ;

fn_call_args: expr | fn_call_args ',' expr;

fn_def
  : "function" '(' ')'
  | "function" '(' fn_def_args ')'
  | "function" '(' fn_def_args ',' ')'
  ;

fn_def_args: IDENT | fn_def_args ',' IDENT;

list
  : '[' ']'
  | '[' list_elems ']'
  | '[' list_elems ';' ']' /* Allow but do not require a trailing semicolon. */
  ;

list_elems : expr | list_elems ';' expr;

block
  : '{' '}'
  | '{' statements '}'
  ;

return: "return" expr;

put: "put" expr;
