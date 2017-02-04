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

expression
  : term_sum { printf("expression "); }
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
  : STRING { printf("string term\n"); }
  | NUMBER
  | COLOR
  | var_ref
  | fn_call
  | fn_def
  | coord
  | '(' expression ')'
  ;

var_ref
  : IDENT
  | var_ref '.' IDENT
  ;

fn_call
  : var_ref '(' ')'
  | var_ref '(' call_args ')'
  ;

call_args
  : expression
  | call_args ',' expression
  ;

fn_def
  : KW_FUNCTION '(' ')'
  | KW_FUNCTION '(' def_args ')'
  ; /* TODO: Block. */

def_args
  : IDENT
  | def_args ',' IDENT
  ;

coord: '(' expression ',' expression ')';
