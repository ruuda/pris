#include <string>
#include <vector>
#include <memory>

struct environment;

struct expr
{
  virtual void evaluate(environment const&) = 0;
};

struct expr_string_lit : public expr
{
  std::string raw_literal;
};

struct expr_num_lit : public expr
{
  std::string raw_literal;
};

struct expr_color_lit : public expr
{
  std::string raw_literal;
};

struct expr_idents : public expr
{
  std::vector<std::string> idents;
};

struct expr_fn_call : public expr
{
  std::vector<std::string> fn_idents;
  std::vector<std::unique_ptr<expr>> args;
};

struct expr_fn_def : public expr
{
  std::vector<std::string> args;
  // TODO: How to store the body? What *is* the body?
};

struct expr_coord : public expr
{
  std::unique_ptr<expr> x;
  std::unique_ptr<expr> y;
};

struct expr_bin_op : public expr
{
  char op;
  std::unique_ptr<expr> lhs;
  std::unique_ptr<expr> rhs;
};
