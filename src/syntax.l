%{
#include <stdio.h>
#include "y.tab.h"
%}

%%
[/][/][^\n]*                 /* Ignore comments */
[ \n]+                       /* Ignore whitespace. */
["]([^"\\]|\\.)*["]          printf("STRING ");
-?[0-9]+(\.[0-9]+)?(w|h|em)? printf("NUMBER ");
#[0-9a-f]{6}                 printf("COLOR ");
(at|put|fit|import)          printf("KEYWORD ");
(namespace|function)         printf("KEYWORD ");
[a-zA-Z][a-zA-Z0-9_]*        printf("IDENT ");
[.]                          printf("DOT ");
[,]                          printf("COMMA ");
[=]                          printf("ASSIGN ");
[{]                          printf("LBRACE ");
[}]                          printf("RBRACE ");
[(]                          printf("LPAR ");
[)]                          printf("RPAR ");
[-+*/^]                      printf("BINOP ");
%%
