#!/usr/bin/env bash

source ~/.bash_profile

PARAMS=()

function add-boolean-param {
  VALUE=$(tmux show -vg @thumbs-$1 2> /dev/null)

  if [[ "${VALUE}" == "1" ]]; then
    PARAMS+=("--$1")
  fi
}

function add-option-param {
  VALUE=$(tmux show -vg @thumbs-$1 2> /dev/null)

  if [[ ${VALUE} ]]; then
    PARAMS+=("--$1=${VALUE}")
  fi
}

function add-multi-param {
  while read -r ITEM_KEY; do
    VALUE=$(tmux show -vg $ITEM_KEY 2> /dev/null)
    PARAMS+=("--$1=${VALUE}")
  done < <(tmux show -g 2> /dev/null | grep thumbs-$1- | cut -d' ' -f1)
}

add-boolean-param "reverse"
add-boolean-param "unique"
add-option-param  "alphabet"
add-option-param  "position"
add-option-param  "fg-color"
add-option-param  "bg-color"
add-option-param  "hint-bg-color"
add-option-param  "hint-fg-color"
add-option-param  "select-fg-color"
add-option-param  "command"
add-option-param  "upcase-command"
add-multi-param   "regexp"
add-boolean-param "contrast"
add-boolean-param "osc52"
add-option-param  "copy-mode-up-key"
add-option-param  "copy-mode-down-key"

# Remove empty arguments from PARAMS.
# Otherwise, they would choke up tmux-thumbs when passed to it.
for i in "${!PARAMS[@]}"; do
  [ -n "${PARAMS[$i]}" ] || unset "PARAMS[$i]"
done

# Find the `tmux-thumbs` binary.
# Prefers `cargo build` output in `target/release`.
# If none exists, try to find a prebuilt binary under `prebuilt/`.
# If still none exists, print an error message and abort.
CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TARGET_RELEASE="/target/release/"

BINARY="${CURRENT_DIR}${TARGET_RELEASE}tmux-thumbs"
if [ ! -e "${BINARY}" ]; then
  PREBUILT_BINARY_NAME="$(uname -s | tr '[:upper:]' '[:lower:]')"
  BINARY="${CURRENT_DIR}/prebuilt/${PREBUILT_BINARY_NAME}"
fi
if [ ! -e "${BINARY}" ]; then
  echo 'Cannot find `tmux-thumb` binary. Did you `cargo build --release`?'
  exit 1
fi

CURRENT_PANE_ID=$(tmux list-panes -F "#{pane_id}:#{?pane_active,active,nope}" | grep active | cut -d: -f1)
NEW_ID=$(tmux new-window -P -d -n "[thumbs]" "${BINARY}" "${PARAMS[@]}" "--tmux-pane=${CURRENT_PANE_ID}")
NEW_PANE_ID=$(tmux list-panes -a | grep ${NEW_ID} | grep --color=never -o '%[0-9]\+')

tmux swap-pane -d -s ${CURRENT_PANE_ID} -t ${NEW_PANE_ID}
