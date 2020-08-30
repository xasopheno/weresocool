setlocal commentstring=--%s%s

setlocal formatoptions-=t formatoptions+=croqnl
" j was only added in 7.3.541, so stop complaints about its nonexistence
silent! setlocal formatoptions+=j

" smartindent will be overridden by indentexpr if filetype indent is on, but
" otherwise it's better than nothing.
setlocal smartindent nocindent

setlocal tabstop=4 shiftwidth=4 softtabstop=4 expandtab
setlocal textwidth=99
setlocal completefunc=syntaxcomplete#Complete
