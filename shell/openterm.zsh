# OpenTerm zsh integration — loaded last inside OpenTerm.app only.

[[ -n "${OPENTERM_PROMPT_READY:-}" ]] && return
export OPENTERM_PROMPT_READY=1

ai-cd() {
  if [ "$#" -ne 1 ]; then
    print -u2 'usage: ai-cd DIRECTORY'
    return 2
  fi
  builtin cd -- "$1"
}

typeset -ga precmd_functions
typeset -g OPENTERM_CMD_RAN=0

_openterm_strip_posh() {
  if (( ${+functions[_omp_do_render]} )); then
    unfunction _omp_do_render 2>/dev/null
  fi
  precmd_functions=(${precmd_functions:#_omp*})
}

_openterm_git_line() {
  ai-context --ansi 2>/dev/null
}

_openterm_draw_prompt() {
  _openterm_strip_posh
  local git_ansi path
  git_ansi=$(_openterm_git_line)
  path=$'%F{252}%~%f'
  if [ -n "$git_ansi" ]; then
    PROMPT=$'%F{238}╭─%f %F{45}◆%f %F{81}%BOpenTerm%b %F{238}·%f '"${path}"$'
%F{238}│%f '"${git_ansi}"$'
%F{238}╰─%f %F{45}❯%f '
    print -Pn "\e]0;OpenTerm · %~\a"
  else
    PROMPT=$'%F{238}╭─%f %F{45}◆%f %F{81}%BOpenTerm%b %F{238}·%f '"${path}"$'
%F{238}╰─%f %F{45}❯%f '
    print -Pn "\e]0;OpenTerm · %~\a"
  fi
  RPROMPT=""
}

_openterm_output_spacing() {
  (( OPENTERM_CMD_RAN )) || return
  OPENTERM_CMD_RAN=0
  print ""
}

if ! (( ${precmd_functions[(Ie)_openterm_output_spacing]} )); then
  precmd_functions+=(_openterm_output_spacing)
fi

if ! (( ${precmd_functions[(Ie)_openterm_draw_prompt]} )); then
  precmd_functions+=(_openterm_draw_prompt)
fi

_openterm_preexec() {
  print ""
  OPENTERM_CMD_RAN=1
}

preexec() {
  _openterm_preexec "$@"
}

_openterm_draw_prompt

if [ -z "${OPENTERM_WELCOMED:-}" ]; then
  export OPENTERM_WELCOMED=1
  print -P ""
  print -P "%F{236}╭─%f %F{81}▐●‿●▌%f  %B%F{255}OpenTerm%f%b  %F{45}agent-first%f %F{236}──────────────────%f"
  print -P "%F{236}│%f  %F{250}ai-rg · ai-read · ai-panel · ai-ask --workspace%f              %F{236}│%f"
  print -P "%F{236}╰──────────────────────────────────────────────────────────────╯%f"
  print -P ""
fi
