pris: lex.yy.c syntax.tab.c
	cc lex.yy.c syntax.tab.c -o $@

syntax.tab.c: src/syntax.y
	bison -d $^

lex.yy.c: src/syntax.l
	lex $^

