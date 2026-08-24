#!/bin/bash
set -euo pipefail

# ============================================================================
# Kinhin Installer
# Installs Kinhin TDD framework to ~/.claude/kinhin
# ============================================================================

# --- Colors (Rich-inspired palette) ---
readonly CYAN='\033[0;36m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[0;33m'
readonly RED='\033[0;31m'
readonly DIM='\033[2m'
readonly BOLD='\033[1m'
readonly NC='\033[0m'

# --- Configuration ---
readonly REPO_URL="https://github.com/daviguides/kinhin.git"
readonly TMP_DIR="/tmp/kinhin-$$"
readonly CLAUDE_DIR="$HOME/.claude"
readonly TARGET_DIR="$CLAUDE_DIR/kinhin"
readonly SOURCE_SUBDIR="kinhin"

# --- Marketplace Configuration ---
readonly MARKETPLACE_URL="https://github.com/daviguides/claude-marketplace.git"
readonly PLUGIN_IDENTIFIER="kinhin@daviguides"

# --- Box width (53 chars between borders) ---
readonly W=53

# --- UI Helpers ---
box_top() {
  printf "${CYAN}╭"
  printf '─%.0s' $(seq 1 $W)
  printf "╮${NC}\n"
}

box_bottom() {
  printf "${CYAN}╰"
  printf '─%.0s' $(seq 1 $W)
  printf "╯${NC}\n"
}

box_empty() {
  printf "${CYAN}│${NC}%${W}s${CYAN}│${NC}\n" ""
}

box_separator() {
  printf "${CYAN}│${DIM}"
  printf '─%.0s' $(seq 1 $W)
  printf "${NC}${CYAN}│${NC}\n"
}

box_text() {
  local text="$1"
  local len=${#text}
  local pad=$((W - 2 - len))
  printf "${CYAN}│${NC}  %s%${pad}s${CYAN}│${NC}\n" "$text" ""
}

status_line() {
  local icon="$1"
  local color="$2"
  local text="$3"
  local len=${#text}
  local pad=$((W - 4 - len))
  printf "${CYAN}│${NC}  ${color}%s${NC} %s%${pad}s${CYAN}│${NC}\n" "$icon" "$text" ""
}

status_ok() { status_line "✓" "$GREEN" "$1"; }
status_warn() { status_line "⚠" "$YELLOW" "$1"; }
status_error() { status_line "✗" "$RED" "$1"; }
status_info() { status_line "→" "$DIM" "$1"; }

spinner() {
  local pid=$1
  local msg="$2"
  local len=${#msg}
  local pad=$((W - 4 - len))
  local spin='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
  local i=0
  while kill -0 "$pid" 2>/dev/null; do
    printf "\r${CYAN}│${NC}  ${CYAN}%s${NC} %s%${pad}s${CYAN}│${NC}" "${spin:i++%10:1}" "$msg" ""
    sleep 0.1
  done
  printf "\r%*s\r" 60 ""
}

# --- Cleanup ---
cleanup() {
  if [ -d "$TMP_DIR" ]; then
    status_info "Cleaning up..."
    rm -rf "$TMP_DIR"
  fi
  box_bottom
  printf "\n"
}
trap cleanup EXIT

# ============================================================================
# Helper Functions
# ============================================================================

print_header() {
  printf "\n"
  box_top
  box_empty
  printf "${CYAN}│${NC}     ${BOLD}●${NC}        ${BOLD}●${NC}                                      ${CYAN}│${NC}\n"
  printf "${CYAN}│${NC}     ↓        ↓     │  Walking Meditation for TDD    ${CYAN}│${NC}\n"
  printf "${CYAN}│${NC}${BOLD}  K  I  N  H  I  N${NC}  │  経行 - Red, Green, Refactor   ${CYAN}│${NC}\n"
  printf "${CYAN}│${NC}     ┴        ┴     │  (v1.2.0)                      ${CYAN}│${NC}\n"
  box_empty
  box_separator
}

check_dependencies() {
  if ! command -v git >/dev/null 2>&1; then
    status_error "git is not installed"
    printf "       Install: https://git-scm.com/downloads\n"
    exit 1
  fi
}

clone_repository() {
  git clone --quiet "$REPO_URL" "$TMP_DIR" 2>/dev/null &
  local pid=$!
  spinner $pid "Cloning repository..."
  wait $pid || {
    status_error "Failed to clone repository"
    exit 1
  }
  status_ok "Repository cloned"
}

validate_source_structure() {
  local src_subdir="$TMP_DIR/$SOURCE_SUBDIR"
  if [ ! -d "$src_subdir" ]; then
    status_error "Expected subfolder not found: $src_subdir"
    exit 1
  fi
}

copy_files() {
  local src_subdir="$TMP_DIR/$SOURCE_SUBDIR"
  mkdir -p "$TARGET_DIR"
  if command -v rsync >/dev/null 2>&1; then
    rsync -a "$src_subdir"/ "$TARGET_DIR"/
  else
    ( set -f; cp -R "$src_subdir"/. "$TARGET_DIR"/ )
  fi
}

install() {
  [ -d "$CLAUDE_DIR" ] || mkdir -p "$CLAUDE_DIR"
  [ -d "$TARGET_DIR" ] && rm -rf "$TARGET_DIR"

  copy_files &
  local pid=$!
  spinner $pid "Installing to ~/.claude/kinhin..."
  wait $pid

  status_ok "Plugin installed to ~/.claude/kinhin"
}

check_claude_cli() {
  if ! command -v claude >/dev/null 2>&1; then
    status_warn "Claude CLI not found, skipping marketplace"
    return 1
  fi
  return 0
}

setup_marketplace() {
  if ! check_claude_cli; then
    return 0
  fi

  claude plugin marketplace add "$MARKETPLACE_URL" >/dev/null 2>&1 &
  local pid=$!
  spinner $pid "Setting up marketplace..."
  wait $pid && status_ok "Marketplace configured" || status_info "Marketplace already configured"

  claude plugin install "$PLUGIN_IDENTIFIER" >/dev/null 2>&1 &
  pid=$!
  spinner $pid "Installing plugin..."
  wait $pid && status_ok "Plugin installed: $PLUGIN_IDENTIFIER" || status_warn "Plugin install failed"
}

box_title() {
  printf "${CYAN}│${NC}  ${GREEN}${BOLD}Installation Complete${NC}                              ${CYAN}│${NC}\n"
}

box_heading() {
  printf "${CYAN}│${NC}  ${BOLD}Available Commands${NC}                                 ${CYAN}│${NC}\n"
}

print_next_steps() {
  box_empty
  box_separator
  box_title
  box_separator
  box_empty
  box_heading
  box_text "/kinhin:load [lang]   → TDD principles"
  box_empty
  box_text "Docs: https://github.com/daviguides/kinhin"
  box_empty
}

# ============================================================================
# Main
# ============================================================================

main() {
  print_header
  check_dependencies
  clone_repository
  validate_source_structure
  install
  setup_marketplace
  print_next_steps
}

main
