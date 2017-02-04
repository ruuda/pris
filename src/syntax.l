%{
// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

#include <stdio.h>
#include "syntax.tab.hpp"

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
