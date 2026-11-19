# lispish [WIP]

## Grammar
```
program    ::= expr*
expr       ::= quote | symbol | number | string | bool | list

quote      ::= '\'' expr
list       ::= '(' expr* ')'

(* tokens matching sign? digit are parsed as number, never symbol *)
symbol     ::= ~(digit | delimiter) (~delimiter)*

number     ::= sign? ( digit+ | digit+ '.' digit* | digit* '.' digit+ ) exp?
exp        ::= [eE] sign? digit+
sign       ::= [+-]
digit      ::= [0-9]

string     ::= '"' ( '\\' . | [^"\\] )* '"'
bool       ::= '#t' | '#f' (* takes precedence over symbol *)

comment    ::= ';' [^\n]*
whitespace ::= [\t\n\f\r ] (* or any unicode whitespace *)
delimiter  ::= [()';"] | whitespace
```

## TODO
- [ ] Tail call optimization
- [ ] Macros
- [ ] Better prelude
- [ ] String manipulation builtins
- [ ] Put WASM in web worker for web REPL
- [ ] CLI
- [ ] File input
- [ ] Formal/spec-compliant grammar
