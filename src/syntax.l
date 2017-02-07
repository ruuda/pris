%{
// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

#include <stdio.h>
#include "syntax.tab.hpp"

#undef YY_DECL
#define YY_DECL yy::parser::symbol_type yyFlexLexer::yylex()

%}

%option c++
%option noyywrap

%%
[/][/][^\n]*                 /* Ignore comments */;
[ \n]+                       /* Ignore whitespace. */;
["]([^"\\]|\\.)*["]          return yy::parser::token::STRING;
-?[0-9]+(\.[0-9]+)?(w|h|em)? return yy::parser::token::NUMBER;
#[0-9a-f]{6}                 return yy::parser::token::COLOR;
at                           return yy::parser::token::KW_AT;
fit                          return yy::parser::token::KW_FIT;
function                     return yy::parser::token::KW_FUNCTION;
import                       return yy::parser::token::KW_IMPORT;
put                          return yy::parser::token::KW_PUT;
[a-zA-Z][a-zA-Z0-9_]*        return yy::parser::token::IDENT;
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
