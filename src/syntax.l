%{
#include <stdio.h>
%}

%%
-?[0-9]+(\.[0-9]+)?   printf("NUMBER ");
[/][/][^\n]*          printf("COMMENT ");
["]([^"\\]|\\.)*["]   printf("STRING ");
[a-zA-Z][a-zA-Z0-9-]* printf("IDENT ");
[}{)(.:]              printf("SYMBOL ");
[ ][-+*/^][ ]         printf("BINOP ");
\n                    printf("NEWLINE ");
[ ]+                  /* Ignore spaces. */
%%
