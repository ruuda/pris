%{
#include <stdio.h>
#include "syntax.tab.h"
%}

%%
[/][/][^\n]*                 /* Ignore comments */;
[ \n]+                       /* Ignore whitespace. */;
["]([^"\\]|\\.)*["]          return STRING;
-?[0-9]+(\.[0-9]+)?(w|h|em)? return NUMBER;
#[0-9a-f]{6}                 return COLOR;
at                           return KW_AT;
fit                          return KW_FIT;
function                     return KW_FUNCTION;
import                       return KW_IMPORT;
put                          return KW_PUT;
[a-zA-Z][a-zA-Z0-9_]*        return IDENT;
[.]                          return '.';
[,]                          return ',';
[=]                          return '=';
[{]                          return '{';
[}]                          return '}';
[(]                          return '(';
[)]                          return ')';
[\^]                         return '^';
[*]                          return '*';
[/]                          return '/';
[+]                          return '+';
[-]                          return '-';
%%
