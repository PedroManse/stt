# STT

###### This describes what the language *should* be, *not necessarily* what it is

Pronounced as "stah", Stt is a simple scripting language made to be embedded in other rust programs.

It's also made to be as simple as possible to parse and execute.

It's recommended usage is to import a standard libraby and make your own
definitions for easier interoperability with the container software system.
Then, before every run modify the stack to include that 'input' variables, and
after executing the user's script, getting the stack values as output.

It's also possible to add rust bindings to be executable by user script


## Execution pipeline
STT has several modules for breaking up each important step, from reading files
until executing the code

### Tokenizer
Everything must begin with the tokenizer reading a string. It reads the content
character by character and arranges them in blocks of `Token`s and specify
where their text is taken from within the string.

It is able to cluster character into:

Token name    | Description
--------------|-----------
Ident         | Identifier of user function, builtin or function argument
Str           | A string
Number        | A positive natural number
Keyword       | A raw keyword
FnArgs        | Text within brackets
Block         | Tokens within curly brackets
EndOfBlock    | End of text or a closing curly bracket
IncludedBlock | Not created by tokenizer


### Preprocessor
The tokenizer and preprocessor work together to assemble all the tokens needed to evaluate the code.

The preprocessor's job is to search the original tokens for specific keyword,
like `include` and `pragma` and to modify the code or state accordingly.

When the preprocessor finds a `pragma` keyword it changes it's own state or
variables. Depending on the preprocessor's state it can ignore all other tokens
to enable conditional inclusion of code or safeguard against multiple
inclusions of the same files

When the preprocessor finds a `include` keyword, that file is read, tokenized
and it's tokens are included in a `IncludedBlock` token. However during the
preprocessing step of an included file, the `pragma` variables are passed down
to the included program's preprocessor. The included files' modifications to
`pragma` variables are also passed up to the file that included them

### Parser
The parser is responsible for joining the tokens into executable expressions
and updating their spans over their source files.

#### Technical detail
There are many tokens with implicit endings, like `(ifs)` who end with any non-code block token when searching for check blocks,
or `(switch)` tokens that end with tokens that aren't immediates or code blocks.

These tokens must add their parsed expression to the output and `unget` the last token, to re-parse it with a different state.

### Execution
###### the runtime module
The expression executioner handles every runtime element and the definition of every builtin

The context of execution is preserved after all the code is ran. So it is possible to reuse the `runtime::Context` and even to utilize previously defined functions.

The recommended usage for context reusage is to send variables to STT code as
values in the stack and to clear the stack after every execution, since a faulty
program could polute another's input.
