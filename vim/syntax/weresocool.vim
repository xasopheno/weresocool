if exists("b:current_syntax")
    finish
endif

echom "Welcome to WereSoCool"
syntax keyword wscKeyword AsIs Tm Ta PanM PanA Gain Length
syntax keyword wscGroup Seq Overlay Sequence
syntax keyword wscEffect ModulateBy FitLength Reverse Invert 
syntax keyword wscO O
syntax keyword wscRepeat Repeat
syntax keyword wscKeyword AD Portamento
syntax match wscOperator "\v\|"
syntax match wscOperator "\v\#"
syntax match wscOperator "\v\$"
syntax match wscOperator "\v\^"
syntax match wscOperator "\v\>"
syntax match wscComma "\v\,"
syntax match wscStructure "\v\{"
syntax match wscStructure "\v\}"
syntax match wscStructure "\v\="
syntax match wscMain "\v\:"
syntax keyword wscMain main 
syntax match wscSpecialChar "\v\("
syntax match wscSpecialChar "\v\)"
syntax match wscSpecialChar "\v\["
syntax match wscSpecialChar "\v\]"
syntax match wscBoolean "\v\/"
syntax match wscNumber "\v<\d+>"
syntax match wscZero "\v\0"
syn region wscCommentLine start="--"  end="$"


highlight link wscOperator Operator
highlight link wscKeyword Keyword
highlight link wscStructure Repeat
highlight link wscSpecial SpecialChar
highlight link wscSpecialChar String
highlight link wscNumber Character
highlight link wscBoolean Boolean
highlight link wscCommentLine Comment
highlight link wscMain Todo
highlight link wscComma Identifier
highlight link wscRepeat Structure
highlight link wscGroup SignifySignChange
highlight link wscEffect SignifySignAdd
highlight link wscO Keyword
highlight link wscZero MoreMsg


let b:current_syntax = "weresocool"
