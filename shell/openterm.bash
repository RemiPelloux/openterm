# OpenTerm Bash integration. Source this file from ~/.bashrc.

ai-cd() {
  if [ "$#" -ne 1 ]; then
    printf '%s\n' 'usage: ai-cd DIRECTORY' >&2
    return 2
  fi
  builtin cd -- "$1"
}

_openterm_draw_prompt() {
  local git_ansi hud hud_file
  git_ansi=$(ai-context --ansi 2>/dev/null)
  hud="ai · idle"
  hud_file="${OPENTERM_DATA_DIR:-$HOME/Library/Application Support/OpenTerm}/hud"
  if [ -r "$hud_file" ]; then
    IFS= read -r hud < "$hud_file"
  fi
  if [ -n "$git_ansi" ]; then
    PS1=$'\033[38;5;238m╭─\033[0m \033[38;5;45m◆\033[0m \033[38;5;81m\033[1mOpenTerm\033[0m \033[38;5;238m·\033[0m \033[38;5;252m\w\033[0m \033[38;5;238m·\033[0m \033[38;5;45m'"${hud}"$'\033[0m\n\033[38;5;238m│\033[0m '"${git_ansi}"$'\n\033[38;5;238m╰─\033[0m \033[38;5;45m❯\033[0m '
  else
    PS1=$'\033[38;5;238m╭─\033[0m \033[38;5;45m◆\033[0m \033[38;5;81m\033[1mOpenTerm\033[0m \033[38;5;238m·\033[0m \033[38;5;252m\w\033[0m \033[38;5;238m·\033[0m \033[38;5;45m'"${hud}"$'\033[0m\n\033[38;5;238m╰─\033[0m \033[38;5;45m❯\033[0m '
  fi
  printf '\033]0;OpenTerm · %s · %s\007' "$PWD" "$hud"
}

_openterm_output_spacing() {
  if [ "${OPENTERM_CMD_RAN:-0}" = 1 ]; then
    printf '\n'
    OPENTERM_CMD_RAN=0
  fi
}

if [ -z "${OPENTERM_BASH_PROMPT_INSTALLED:-}" ]; then
  export OPENTERM_BASH_PROMPT_INSTALLED=1
  OPENTERM_ORIGINAL_PROMPT_COMMAND=${PROMPT_COMMAND:-}
  PROMPT_COMMAND='_openterm_output_spacing;_openterm_draw_prompt'
  if [ -n "$OPENTERM_ORIGINAL_PROMPT_COMMAND" ]; then
    PROMPT_COMMAND="$PROMPT_COMMAND;$OPENTERM_ORIGINAL_PROMPT_COMMAND"
  fi
  OPENTERM_CMD_RAN=0
  trap '_openterm_bash_preexec "$BASH_COMMAND"' DEBUG
fi

_openterm_bash_preexec() {
  case "$1" in
    _openterm_*|trap*|PROMPT_COMMAND=*) return ;;
  esac
  if [ "${OPENTERM_CMD_RAN:-0}" != 1 ]; then
    printf '\n'
    OPENTERM_CMD_RAN=1
  fi
}

if [ -z "${OPENTERM_WELCOMED:-}" ]; then
  export OPENTERM_WELCOMED=1
  printf '\n\033[48;5;236m\033[38;5;238m ╭─\033[0m \033[38;5;81m▐●‿●▌\033[0m  \033[1m\033[38;5;255mOpenTerm\033[0m  \033[38;5;45magent-first\033[0m \033[38;5;238m──────────────────\033[0m\n'
  printf '\033[48;5;236m\033[38;5;238m │\033[0m  \033[38;5;250mai-rg · ai-read · ai-panel · ai-ask --workspace\033[0m              \033[38;5;238m│\033[0m\n'
  printf '\033[48;5;236m\033[38;5;238m ╰──────────────────────────────────────────────────────────────\033[0m\033[0m\n\n'
fi
