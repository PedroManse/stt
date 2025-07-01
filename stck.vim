" true/false
syn keyword Boolean true false
hi def link Boolean Identifier

" get/set
syn keyword VarOp get set
hi def link VarOp Keyword

" (while) (ifs)
syn match OpenParam "(" nextgroup=Keyword,Include,FnDef,Pragma
syn keyword Keyword while ifs contained
syn match CloseParam ")" contained
hi def link     Keyword        Keyword

" (pragma)
syn match Pragma "pragma " nextgroup=PragmaCommand contained
hi def link     Pragma        Keyword
hi def link     PragmaCommand        Number
syn match PragmaCommand "\(\(if not\|if\|set\|unset\) [^)]\+\|\(else\|end if\)\)" contained nextgroup=CloseParam

" (include)
syn keyword Include include nextgroup=IncludeFilePath contained
hi def link     Include        Include

syn match      IncludeFilePath  " [a-zA-Z/]\+\(.stck\)\?" contained nextgroup=CloseParam
hi def link IncludeFilePath Comment

" Ident
syn match IdentStart "[a-zA-Z+_\-%!?$=*&<>≃,:~@]" nextgroup=Ident
syn match Ident "[a-zA-Z+_\-%!?$=*&<>≃,:~@./\']" nextgroup=Ident
"hi def link IdentStart Identifier
"hi def link Ident Identifier

" char 'c'
syn match CharStart "'" nextgroup=CharCont,CharSpecial,CharWrong
syn match CharCont "[^\\\']" nextgroup=CharClose,CharError contained
syn match CharSpecial "\\" nextgroup=CharSpecialChar,CharSpecialCharError contained
syn match CharSpecialChar "\(n\|\\\|'\)" contained nextgroup=CharClose,CharError
syn match CharSpecialCharError '[^n\\\']' contained nextgroup=CharError,CharClose
syn match CharClose "'" contained
syn match CharError '[^\']' contained nextgroup=CharError,CharClose

hi def link CharCont Comment
hi def link CharClose Comment
hi def link CharStart Comment
hi def link CharSpecial Comment
hi def link CharSpecialChar Keyword
hi def link CharSpecialCharError Error
hi def link CharError Error

" (fn)
syn keyword FnDef fn nextgroup=FnDefStart,FnDefScope contained
" " fn scope
syn match FnDefScope "\(*\|-\)" nextgroup=FnDefStart contained

" " close param
syn match FnDefStart ") " contained nextgroup=FnDefArgsStartEmpty,FnDefArgsAllStack,FnDefArgsStartArgs

" " allstack as arg
syn match FnDefArgsAllStack "\*" contained

" " parse [...] args
" FnDefArgsStartArgs doesn't need to be contained because (fn) and closures
" can start this
syn match FnDefArgsStartArgs "\[\(\s\|\\n\)*" nextgroup=FnDefArgsArg,FnDefArgsEnd
syn match FnDefArgsStartEmpty "\[\s*\]" contained
syn match FnDefArgsArg "\<\w\+" nextgroup=FnDefArgsArgType,FnDefArgsArg contained
syn match FnDefArgsArgType "<\s*" nextgroup=SFTC,FnDefArgsArgTypeInsSimple contained
syn match FnDefArgsArgTypeInsEnd "\s*>\s*" nextgroup=FnDefArgsArg,FnDefArgsEnd contained
syn match FnDefArgsEnd "\]" contained

" " highlight
hi def link     FnDefArgsStartEmpty Delimiter
hi def link     FnDefScope Delimiter
hi def link     FnDef        Keyword
hi def link     FnDefArgs        Keyword
hi def link     FnDefArgsAllStack        Keyword
hi def link     FnDefArgsArg        Keyword
hi def link     FnDefArgsArgType Delimiter
hi def link     FnDefArgsArgTypeInsEnd Delimiter

" Typing matches
syn keyword FnDefArgsArgTypeInsSimple char string str num bool nextgroup=FnDefArgsArgTypeInsEnd contained
syn match SFTC "array" nextgroup=FnDefArgsArgTypeInsEnd contained
syn match SFTC "array<\w\+>" nextgroup=FnDefArgsArgTypeInsEnd contained
syn match SFTC "fn" nextgroup=FnDefArgsArgTypeInsEnd contained
syn match SFTC "fn<\w\+>" nextgroup=FnDefArgsArgTypeInsEnd contained
syn match SFTC "fn<\w\+>\s*<\w\+>" nextgroup=FnDefArgsArgTypeInsEnd contained

hi def link     SFTC Number
hi def link     FnDefArgsArgTypeInsSimple Number

" string
syn region      String            start=+"+ end=+"+
hi def link     String            String

" number
syn match Number "\<\(0\|[1-9][0-9]*\)\>"
hi def link Number Number

" method division bla$bla$bla
syn match SubFnName "\>\$\<"
hi def link SubFnName Delimiter

" panic-able functions or the '!' panic function
syn match MayPanic "!"
hi def link MayPanic Error

" comment
syn match Comment "#.*"
hi def link Comment Comment


