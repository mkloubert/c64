# Language Grammar Specification

This document defines the complete grammar of the C64 language in EBNF notation.

## Notation

```
=           definition
|           alternation
[ ... ]     optional (0 or 1)
{ ... }     repetition (0 or more)
( ... )     grouping
" ... "     terminal string
' ... '     terminal character
/* ... */   comment
```

---

## Lexical Grammar

### Whitespace and Comments

```ebnf
WHITESPACE      = ' ' | '\t' ;
NEWLINE         = '\n' | '\r\n' ;
COMMENT         = '#' { any_char_except_newline } NEWLINE ;
INDENT          = WHITESPACE WHITESPACE WHITESPACE WHITESPACE ;  /* 4 spaces */
```

### Identifiers

```ebnf
IDENTIFIER      = LETTER { LETTER | DIGIT | '_' } ;
LETTER          = 'a'..'z' | 'A'..'Z' | '_' ;
DIGIT           = '0'..'9' ;
```

### Literals

```ebnf
INTEGER_LIT     = DECIMAL_LIT | HEX_LIT | BINARY_LIT ;
DECIMAL_LIT     = DIGIT { DIGIT } ;
HEX_LIT         = '$' HEX_DIGIT { HEX_DIGIT } ;
BINARY_LIT      = '%' BIN_DIGIT { BIN_DIGIT } ;
HEX_DIGIT       = DIGIT | 'a'..'f' | 'A'..'F' ;
BIN_DIGIT       = '0' | '1' ;

CHAR_LIT        = "'" ( CHAR | ESCAPE_SEQ ) "'" ;
STRING_LIT      = '"' { CHAR | ESCAPE_SEQ } '"' ;
CHAR            = /* any character except ', ", \, newline */ ;
ESCAPE_SEQ      = '\\' ( 'n' | 'r' | 't' | '\\' | '"' | "'" | '0' ) ;

BOOL_LIT        = 'true' | 'false' ;
```

### Keywords

```ebnf
KEYWORD         = 'byte' | 'word' | 'sbyte' | 'sword' | 'bool' | 'string'
                | 'const' | 'def' | 'return'
                | 'if' | 'elif' | 'else'
                | 'while' | 'for' | 'in' | 'to' | 'downto'
                | 'break' | 'continue'
                | 'and' | 'or' | 'not'
                | 'true' | 'false'
                | 'pass' ;
```

### Operators and Punctuation

```ebnf
OPERATOR        = '+' | '-' | '*' | '/' | '%'
                | '==' | '!=' | '<' | '>' | '<=' | '>='
                | '&' | '|' | '^' | '~' | '<<' | '>>'
                | '=' | '+=' | '-=' | '*=' | '/=' | '%='
                | '&=' | '|=' | '^=' | '<<=' | '>>=' ;

PUNCTUATION     = '(' | ')' | '[' | ']' | ':' | ',' | '->' ;
```

---

## Syntactic Grammar

### Program Structure

```ebnf
program         = { statement } ;

statement       = const_decl
                | var_decl
                | func_def
                | simple_stmt NEWLINE
                | compound_stmt ;

simple_stmt     = assignment
                | func_call
                | return_stmt
                | break_stmt
                | continue_stmt
                | pass_stmt ;

compound_stmt   = if_stmt
                | while_stmt
                | for_stmt ;
```

### Declarations

```ebnf
const_decl      = 'const' IDENTIFIER '=' expr NEWLINE ;

var_decl        = type IDENTIFIER [ '=' expr ] NEWLINE
                | type IDENTIFIER '[' INTEGER_LIT ']' [ '=' array_init ] NEWLINE ;

type            = 'byte' | 'word' | 'sbyte' | 'sword' | 'bool' | 'string' ;

array_init      = '[' expr { ',' expr } ']' ;
```

### Function Definition

```ebnf
func_def        = 'def' IDENTIFIER '(' [ param_list ] ')' [ '->' type ] ':' NEWLINE
                  INDENT block ;

param_list      = param { ',' param } ;
param           = type IDENTIFIER ;

block           = { INDENT statement } ;
```

### Control Flow

```ebnf
if_stmt         = 'if' expr ':' NEWLINE INDENT block
                  { 'elif' expr ':' NEWLINE INDENT block }
                  [ 'else' ':' NEWLINE INDENT block ] ;

while_stmt      = 'while' expr ':' NEWLINE INDENT block ;

for_stmt        = 'for' IDENTIFIER 'in' expr ( 'to' | 'downto' ) expr ':' NEWLINE
                  INDENT block ;
```

### Statements

```ebnf
assignment      = lvalue '=' expr
                | lvalue '+=' expr
                | lvalue '-=' expr
                | lvalue '*=' expr
                | lvalue '/=' expr
                | lvalue '%=' expr
                | lvalue '&=' expr
                | lvalue '|=' expr
                | lvalue '^=' expr
                | lvalue '<<=' expr
                | lvalue '>>=' expr ;

lvalue          = IDENTIFIER [ '[' expr ']' ] ;

func_call       = IDENTIFIER '(' [ arg_list ] ')' ;
arg_list        = expr { ',' expr } ;

return_stmt     = 'return' [ expr ] ;
break_stmt      = 'break' ;
continue_stmt   = 'continue' ;
pass_stmt       = 'pass' ;
```

### Expressions

```ebnf
expr            = or_expr ;

or_expr         = and_expr { 'or' and_expr } ;
and_expr        = not_expr { 'and' not_expr } ;
not_expr        = 'not' not_expr | comparison ;

comparison      = bitor_expr { comp_op bitor_expr } ;
comp_op         = '==' | '!=' | '<' | '>' | '<=' | '>=' ;

bitor_expr      = bitxor_expr { '|' bitxor_expr } ;
bitxor_expr     = bitand_expr { '^' bitand_expr } ;
bitand_expr     = shift_expr { '&' shift_expr } ;

shift_expr      = add_expr { ( '<<' | '>>' ) add_expr } ;
add_expr        = mul_expr { ( '+' | '-' ) mul_expr } ;
mul_expr        = unary_expr { ( '*' | '/' | '%' ) unary_expr } ;

unary_expr      = ( '-' | '~' ) unary_expr | primary ;

primary         = INTEGER_LIT
                | CHAR_LIT
                | STRING_LIT
                | BOOL_LIT
                | IDENTIFIER [ '[' expr ']' ]
                | func_call
                | '(' expr ')' ;
```

---

## Indentation Rules

1. **Indentation unit**: 4 spaces (tabs are not allowed)
2. **Block start**: After `:` at end of line, next lines must be indented
3. **Block end**: First line with less indentation ends the block
4. **Nesting**: Each nesting level adds 4 more spaces
5. **Empty lines**: Ignored, do not affect indentation

### Example

```
def example():          # Level 0
    if condition:       # Level 1 (4 spaces)
        do_something()  # Level 2 (8 spaces)
        if nested:      # Level 2
            inner()     # Level 3 (12 spaces)
    else:               # Level 1
        other()         # Level 2
```

---

## Grammar Notes

### Left-Recursive Elimination

The grammar is written to avoid left recursion, making it suitable for recursive descent parsing.

### Operator Associativity

- Binary operators are left-associative
- Unary operators are right-associative
- Assignment operators are right-associative

### Precedence

See `02-operators.md` for the complete operator precedence table.

### Ambiguity Resolution

- `else` binds to the nearest `if`
- Function call has higher precedence than array access
- Parentheses can be used to override precedence

---

## Example Parse Tree

For the expression `a + b * c`:

```
expr
└── add_expr
    ├── mul_expr
    │   └── primary (a)
    ├── '+'
    └── mul_expr
        ├── primary (b)
        ├── '*'
        └── primary (c)
```

Result: `a + (b * c)` (multiplication has higher precedence)
