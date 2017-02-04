%{
#include <stdio.h>
#include <string.h>

int yydebug=1;
 
void yyerror(const char *str)
{
  fprintf(stderr,"syntax error: %s\n",str);
}
 
int yywrap()
{
  return 1;
} 
  
int main()
{
  yyparse();
} 

%}

%token STRING NUMBER COLOR KW_AT KW_FIT KW_FUNCTION KW_IMPORT KW_NAMESPACE KW_PUT IDENT

%%

toplevels
  : /* empty */
  | toplevels toplevel
  ;

toplevel
  : KW_IMPORT idents   { printf("import\n"); }
  | assignment         { printf("assignment\n"); }
  | '{' statements '}' { printf("frame\n"); }
  ;

assignment: IDENT '=' expression;

statements
  : /* empty */
  | statements statement
  ;

statement
  : assignment
  | put_at
  ;

put_at
  : KW_PUT expression KW_AT expression
  | KW_AT expression KW_PUT expression
  ;

expression
  : term_sum
  | term_sum KW_FIT term_sum
  ;

term_sum
  : term_prod
  | term_sum '+' term_prod
  | term_sum '-' term_prod
  ;

term_prod
  : term_exp
  | term_prod '*' term_exp
  | term_prod '/' term_exp
  ;

term_exp
  : term
  | term '^' term
  ;

term
  : STRING
  | NUMBER
  | COLOR
  | idents
  | fn_call
  | fn_def
  | coord
  | '{' statements '}'
  | '(' expression ')'
  ;

idents
  : IDENT
  | idents '.' IDENT
  ;

fn_call
  : idents '(' ')'
  | idents '(' call_args ')'
  ;

call_args
  : expression
  | call_args ',' expression
  ;

fn_def
  : KW_FUNCTION '(' ')' fn_body
  | KW_FUNCTION '(' def_args ')' fn_body
  ;

def_args
  : IDENT
  | def_args ',' IDENT
  ;

fn_body: '{' statements '}';

coord: '(' expression ',' expression ')';
