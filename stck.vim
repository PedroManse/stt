" true/false
syn keyword SttBoolean true false
hi def link SttBoolean Identifier

" get/set
syn keyword SttVarOp get set
hi def link SttVarOp Keyword

" (while) (ifs)
syn match SttOpenParam "(" nextgroup=SttKeyword,SttInclude,SttFnDef,SttPragma
syn keyword SttKeyword while ifs contained
syn match SttCloseParam ")" contained
hi def link     SttKeyword        Keyword

" (pragma)
syn match SttPragma "pragma " nextgroup=SttPragmaCommand contained
hi def link     SttPragma        Keyword
hi def link     SttPragmaCommand        Number
syn match SttPragmaCommand "\(\(if not\|if\|set\|unset\) [^)]\+\|\(else\|end if\)\)" contained nextgroup=SttCloseParam

" (include)
syn keyword SttInclude include nextgroup=SttIncludeFilePath contained
hi def link     SttInclude        Include

syn match      SttIncludeFilePath  " [a-zA-Z/]\+\(.stt\)\?" contained nextgroup=SttCloseParam
hi def link SttIncludeFilePath Comment

" Ident
syn match SttIdentStart "[a-zA-Z+_\-%!?$=*&<>≃,:~@]" nextgroup=SttIdent
syn match SttIdent "[a-zA-Z+_\-%!?$=*&<>≃,:~@./\']" nextgroup=SttIdent
"hi def link SttIdentStart Identifier
"hi def link SttIdent Identifier

" char 'c'
syn match SttCharStart "'" nextgroup=SttCharCont,SttCharSpecial,SttCharWrong
syn match SttCharCont "[^\\\']" nextgroup=SttCharClose,SttCharError contained
syn match SttCharSpecial "\\" nextgroup=SttCharSpecialChar,SttCharSpecialCharError contained
syn match SttCharSpecialChar "\(n\|\\\|'\)" contained nextgroup=SttCharClose,SttCharError
syn match SttCharSpecialCharError '[^n\\\']' contained nextgroup=SttCharError,SttCharClose
syn match SttCharClose "'" contained
syn match SttCharError '[^\']' contained nextgroup=SttCharError,SttCharClose

hi def link SttCharCont Comment
hi def link SttCharClose Comment
hi def link SttCharStart Comment
hi def link SttCharSpecial Comment
hi def link SttCharSpecialChar Keyword
hi def link SttCharSpecialCharError Error
hi def link SttCharError Error

" (fn)
syn keyword SttFnDef fn nextgroup=SttFnDefStart,SttFnDefScope contained
" " fn scope
syn match SttFnDefScope "\(*\|-\)" nextgroup=SttFnDefStart contained

" " close param
syn match SttFnDefStart ") " contained nextgroup=SttFnDefArgsStartEmpty,SttFnDefArgsAllStack,SttFnDefArgsStartArgs

" " allstack as arg
syn match SttFnDefArgsAllStack "\*" contained

" " parse [...] args
" SttFnDefArgsStartArgs doesn't need to be contained because (fn) and closures
" can start this
syn match SttFnDefArgsStartArgs "\[\(\s\|\\n\)*" nextgroup=SttFnDefArgsArg,SttFnDefArgsEnd
syn match SttFnDefArgsStartEmpty "\[\s*\]" contained
syn match SttFnDefArgsArg "\<\w\+" nextgroup=SttFnDefArgsArgType,SttFnDefArgsArg contained
syn match SttFnDefArgsArgType "<\s*" nextgroup=SFTC,SttFnDefArgsArgTypeInsSimple contained
syn match SttFnDefArgsArgTypeInsEnd "\s*>\s*" nextgroup=SttFnDefArgsArg,SttFnDefArgsEnd contained
syn match SttFnDefArgsEnd "\]" contained

" " highlight
hi def link     SttFnDefArgsStartEmpty Delimiter
hi def link     SttFnDefScope Delimiter
hi def link     SttFnDef        Keyword
hi def link     SttFnDefArgs        Keyword
hi def link     SttFnDefArgsAllStack        Keyword
hi def link     SttFnDefArgsArg        Keyword
hi def link     SttFnDefArgsArgType Delimiter
hi def link     SttFnDefArgsArgTypeInsEnd Delimiter

" Typing matches
syn keyword SttFnDefArgsArgTypeInsSimple char string str num bool nextgroup=SttFnDefArgsArgTypeInsEnd contained
syn match SFTC "array" nextgroup=SttFnDefArgsArgTypeInsEnd contained
syn match SFTC "array<\w\+>" nextgroup=SttFnDefArgsArgTypeInsEnd contained
syn match SFTC "fn" nextgroup=SttFnDefArgsArgTypeInsEnd contained
syn match SFTC "fn<\w\+>" nextgroup=SttFnDefArgsArgTypeInsEnd contained
syn match SFTC "fn<\w\+>\s*<\w\+>" nextgroup=SttFnDefArgsArgTypeInsEnd contained

hi def link     SFTC Number
hi def link     SttFnDefArgsArgTypeInsSimple Number

" string
syn region      SttString            start=+"+ end=+"+
hi def link     SttString            String

" number
syn match SttNumber "\<\(0\|[1-9][0-9]*\)\>"
hi def link SttNumber Number

" method division bla$bla$bla
syn match SttSubFnName "\>\$\<"
hi def link SttSubFnName Delimiter

" panic-able functions or the '!' panic function
syn match SttMayPanic "!"
hi def link SttMayPanic Error

" comment
syn match SttComment "#.*"
hi def link SttComment Comment


