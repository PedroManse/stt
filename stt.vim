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

" (fn)
syn keyword SttFnDef fn nextgroup=SttFnDefStart,SttFnDefScope contained
" " fn scope
syn match SttFnDefScope "\(*\|-\)" nextgroup=SttFnDefStart contained

" " close param
syn match SttFnDefStart ") " contained nextgroup=SttFnDefArgsStartEmpty,SttFnDefArgsAllStack,SttFnDefArgsStartArgs

" " allstack as arg
syn match SttFnDefArgsAllStack "\*" contained

" " parse [...] args
syn match SttFnDefArgsStartArgs "\[\s*" contained nextgroup=SttFnDefArgsArg,SttFnDefArgsEnd
syn match SttFnDefArgsStartEmpty "\[\s*\]" contained
syn match SttFnDefArgsArg "\<\w\+\> \?" nextgroup=SttFnDefArgsArg,SttFnDefArgsEnd contained
syn match SttFnDefArgsEnd "\]" contained

" " highlight
hi def link     SttFnDefArgsStartEmpty Delimiter
hi def link     SttFnDefScope Delimiter
hi def link     SttFnDef        Keyword
hi def link     SttFnDefArgs        Keyword
hi def link     SttFnDefArgsAllStack        Keyword
hi def link     SttFnDefArgsArg        Keyword

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


