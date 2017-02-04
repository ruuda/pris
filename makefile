pris: lex.yy.c
	cc $^ -o pris -lfl

lex.yy.c: src/syntax.l
	lex $^

