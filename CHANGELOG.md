# Changelog

## 0.3.0
* Added interpreter as binary, made crate into lib (#50)
* Renamed STT to Stck
* Split lib.rs into internals, error and types (#82 #83)
* Removed `StrVecIntoStringVec` trait from tests, no longer useful
* Tests for type checkers
* Better control of code execution (#46)
* Raw formats of api, to use each pipeline step discretely (#46)
* Rust hook (#47)
* Fixed tokenizer EOF bug (#63)

### Typing #5
* Both input and output argument lists' names are optional
* Allowed for extra argument list in function definition for output types
* Parsed typed input arguments
* Change FnName & ArgName to be simple String aliases
* stck.vim update for simple types and partial support for composite types
* Runtime execution of type checkers
* Type system for simple and composite types
* Test typing of closures
* Added nested typing
* Added generics and confined generics (defined generics)

## 0.2.0
* Restructure crate as lib and publish first version (#51)
* Add rust hook function as callable in stck script (#47)
* GPLv3 as license (#36)
* Closures
* `!` Keyword to bubble up or unpack result
* Disallow parsing of closures with zero argunents (#12)
* Use @ prefix as keyword to make function into closure (#1)
* Char variable type (#4)
* Remove need for fake `}` in end of tokenizer (#15)
* Print original format string on `%%` error (#16)
* Make closures capture function arguments (#21)
* Removed all uses of `TodoErr` (#22)
* Allow `'` as ident token (#23)
* Wrote README (#31)
* Renamed Stt to Stck (#26)
* Nix derivation (#35)

## 0.1.0
* Basic functions for minimal language interpreter
* Simple GitHub Actions script

