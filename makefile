pris: lex.yy.cc syntax.tab.cpp
	$(CXX) -std=c++14 lex.yy.cc syntax.tab.cpp -o $@

syntax.tab.c: src/syntax.ypp
	bison -d $^

lex.yy.cc: src/syntax.l
	lex --c++ $^

