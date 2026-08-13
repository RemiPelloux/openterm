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
typeset -g OPENTERM_GIT_LINE=""
typeset -g OPENTERM_GIT_PWD=""
typeset -g OPENTERM_GIT_FD=0

if (( ${+functions[_omp_do_render]} )); then
  unfunction _omp_do_render 2>/dev/null
fi
precmd_functions=(${precmd_functions:#_omp*})

# Renders from the cached git line only. Never forks, so drawing a prompt costs
# nothing regardless of how large the repository is.
_openterm_draw_prompt() {
  local path=$'%F{252}%~%f'
  local head=$'%F{238}╭─%f %F{45}◆%f %F{81}%BOpenTerm%b %F{238}·%f '"${path}"
  if [ -n "$OPENTERM_GIT_LINE" ]; then
    PROMPT="${head}"$'
%F{238}│%f '"${OPENTERM_GIT_LINE}"$'
%F{238}╰─%f %F{45}❯%f '
  else
    PROMPT="${head}"$'
%F{238}╰─%f %F{45}❯%f '
  fi
  RPROMPT=""
  print -Pn "\e]0;OpenTerm · %~\a"
}

_openterm_git_ready() {
  local fd=$1 line=""
  IFS= read -r line <&$fd
  zle -F $fd
  exec {fd}<&-
  OPENTERM_GIT_FD=0
  if [ "$line" != "$OPENTERM_GIT_LINE" ]; then
    OPENTERM_GIT_LINE=$line
    _openterm_draw_prompt
    zle reset-prompt
  fi
}

# Collects git state in the background and repaints once it lands.
_openterm_git_refresh() {
  (( OPENTERM_GIT_FD )) && return
  exec {OPENTERM_GIT_FD}< <(ai-context --ansi 2>/dev/null)
  zle -F $OPENTERM_GIT_FD _openterm_git_ready
}

_openterm_precmd() {
  if [ "$PWD" != "$OPENTERM_GIT_PWD" ]; then
    OPENTERM_GIT_PWD=$PWD
    OPENTERM_GIT_LINE=""
  fi
  _openterm_draw_prompt
  _openterm_git_refresh
}

_openterm_output_spacing() {
  (( OPENTERM_CMD_RAN )) || return
  OPENTERM_CMD_RAN=0
  print ""
}

if ! (( ${precmd_functions[(Ie)_openterm_output_spacing]} )); then
  precmd_functions+=(_openterm_output_spacing)
fi

if ! (( ${precmd_functions[(Ie)_openterm_precmd]} )); then
  precmd_functions+=(_openterm_precmd)
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
