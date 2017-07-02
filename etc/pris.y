%{

%}

%token IDENT COLOR NUMBER STRING

%%

statements: statement | statements statement;

statement
  : import
  | assign
  | return
  | block
  | put_at
  ;

import: "import" idents;

idents: IDENT | idents '.' IDENT;

assign: IDENT '=' expr;

expr: expr_add;

expr_add
  : expr_mul
  | expr_mul '+' expr_add
  | expr_mul '-' expr_add
  | expr_mul '~' expr_add
  ;

expr_mul
  : expr_exp
  | expr_exp '*' expr_mul
  | expr_exp '/' expr_mul
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

put_at
  : "put" expr "at" expr
  | "at" expr "put" expr
  ;

