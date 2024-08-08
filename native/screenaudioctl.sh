#!/bin/bash


LOCAL_INSTALL_SHARE="$HOME/.local/share/pipewire-screenaudio"
LOCAL_INSTALL_LIB="$HOME/.local/lib/pipewire-screenaudio"

SYSTEM_INSTALL_SHARE="/usr/share/pipewire-screenaudio"
SYSTEM_INSTALL_LIB="/usr/lib/pipewire-screenaudio"

USER_MOZILLA_DIR="$HOME/.mozilla"
USER_MOZILLA_DIR_FLATPAK="$HOME/.var/app/org.mozilla.firefox/.mozilla"


if [ -d "$LOCAL_INSTALL_SHARE" ] && [ -d "$LOCAL_INSTALL_LIB" ]; then # Check for local install
  CONNECTOR_BIN_SOURCE_PATH="$LOCAL_INSTALL_LIB/connector-rs"
  NATIVE_MSG_HOST_SOURCE_DIR="$LOCAL_INSTALL_SHARE/native-messaging-hosts"
elif [ -d "$SYSTEM_INSTALL_SHARE" ] && [ -d "$SYSTEM_INSTALL_LIB"]; then # If no local install fallback to system install
  CONNECTOR_BIN_SOURCE_PATH="$SYSTEM_INSTALL_LIB/connector-rs"
  NATIVE_MSG_HOST_SOURCE_DIR="$SYSTEM_INSTALL_SHARE/native-messaging-hosts"
else
	echo "Error: failed to find valid pw-screenaudio install paths"
	exit 1
fi

#TODO parse TOML config and validate everything. if valid override default PATHS


#CONNECTOR_BIN_SOURCE_PATH="$HOME/pipewire-screenaudio/native/connector-rs/target/debug/connector-rs"
#NATIVE_MSG_HOST_SOURCE_DIR="$HOME/pipewire-screenaudio/native/native-messaging-hosts"
#USER_MOZILLA_DIR="$HOME/.mozilla"
#USER_MOZILLA_DIR_FLATPAK="$HOME/.var/app/org.mozilla.firefox/.mozilla"

show_help() {
  echo "Usage: script.sh [action] [options]"
  echo "Actions:"
  echo "  apply                     Apply changes"
  echo "  list-destinations         List destinations"
  echo "  reset                     Delete .mozilla folders, reset changes to firefox flatpak (NOT IMPLEMENTED)"
  echo "Options:"
  echo "  -h, --help                Show help"
  echo "  -v, --verbose             Enable verbose mode"
}


declare -A destination_paths
declare -a destinations

detect_mozilla_dirs() {
  destination_paths=()
  # Check for user-installed Firefox
  if command -v firefox &>/dev/null && [ -d "$USER_MOZILLA_DIR" ]; then
      destinations+=("user")
      destination_paths["user"]="$USER_MOZILLA_DIR"
  fi

    # Check for Firefox installed via flatpak
  if flatpak list | grep -q org.mozilla.firefox && [ -d "$USER_MOZILLA_DIR_FLATPAK" ]; then
      destinations+=("user-flatpak")
      destination_paths["user-flatpak"]="$USER_MOZILLA_DIR_FLATPAK"
  fi
}

detect_applied_mozilla_dirs() {
  destination_paths=()

  if command -v firefox &>/dev/null && [ -d "$USER_MOZILLA_DIR" ]; then
    if [ -f "$USER_MOZILLA_DIR/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json" ]; then
      destinations+=("user")
      destination_paths["user"]="$USER_MOZILLA_DIR"
    fi
  fi

  if flatpak list | grep -q org.mozilla.firefox && [ -d "$USER_MOZILLA_DIR_FLATPAK" ]; then
    if [ -f "$USER_MOZILLA_DIR_FLATPAK/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json" ] || \
       [ -f "$USER_MOZILLA_DIR_FLATPAK/native-messaging-hosts/connector-rs" ]; then
      destinations+=("user-flatpak")
      destination_paths["user-flatpak"]="$USER_MOZILLA_DIR_FLATPAK"
		fi
  fi
}



list_mozilla_dirs() {
  if [ ${#destinations[@]} -eq 0 ]; then
    echo "No .mozilla directories found."
    exit 1
    else
    echo -e "\nFound the following .mozilla directories:\n"
    local i
    for i in "${!destinations[@]}"; do
      local key="${destinations[$i]}"
      echo -e "$((i + 1)). $key:\n   ${destination_paths[$key]}"
     done
  fi
}

select_mozilla_dir() {
  local selection
  echo -en "\nSelect where to apply changes to ([1-${#destinations[@]}] a=all, 0=abort): "

  read -r selection
    # Check for abort condition
  if [[ "$selection" == '0' ]]; then
    echo "Aborting operation"
    exit 0
    # Check for 'a' to apply to all or a valid number
  elif [[ "$selection" == 'a' || ( "$selection" =~ ^[0-9]+$ && "$selection" -ge 1 && "$selection" -le ${#destinations[@]} ) ]]; then
    echo "$selection"
  else
    echo "Invalid selection."
    exit 1
  fi
}


apply_to_mozilla_dir(){
	local mozilla_dir="$1"
	local dest_type="$2"

  if [ -d "$mozilla_dir" ]; then

    echo -e "\ncreating native-messaging-hosts directory in $mozilla_dir"
    mkdir -p $mozilla_dir/native-messaging-hosts

		if [ "$dest_type" = "user-flatpak" ]; then
  		echo -e "\ncopying connector-rs binary to $CONNECTOR_BIN_SOURCE_PATH in $mozilla_dir/native-messaging-hosts/connecter-rs"
  		cp $CONNECTOR_BIN_SOURCE_PATH $mozilla_dir/native-messaging-hosts/

      echo -e "\ngiving org.mozilla.firefox userwide pipewire permissions"
	    flatpak override --user --filesystem=xdg-run/pipewire-0 org.mozilla.firefox

			json_connector_path="/home/$(whoami)/.mozilla/native-messaging-hosts/connector-rs"
		else
			json_connector_path=$CONNECTOR_BIN_SOURCE_PATH
		fi

		echo "copying firefox.json into ${mozilla_dir}/native-messaging-hosts"
	  jq --arg newpath "$json_connector_path" '.path = $newpath' "$NATIVE_MSG_HOST_SOURCE_DIR/firefox.json" \
    > "$mozilla_dir/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json"

  else
    echo "Mozilla directory not found: $mozilla_dir"
    exit 1
  fi
}

reset_mozilla_dir(){
  local mozilla_dir=$1
  local dest_type=$2
	# WIP
  if [ -d "$mozilla_dir" ]; then
    if [ "$dest_type" = "user-flatpak" ]; then
      echo -e "\nresetting pipewire permissions org.mozilla.firefox\n"
      flatpak override --reset org.mozilla.firefox
	fi

  echo -e "\ndeleting native-messaging-hosts directory in $mozilla_dir\n"
  rm -rf "$mozilla_dir/native-messaging-hosts"

  else
     echo "Installation path not found: $mozilla_dir"
     exit 1
  fi
}

list_action(){
  if [ $installed -eq 1 ]; then
    detect_applied_mozilla_dirs
  else
    detect_mozilla_dirs
	fi
  list_mozilla_dirs
}


apply_action() {
  detect_mozilla_dirs
  list_mozilla_dirs
  select_mozilla_dir

  if [ "$selection" = 'a' ]; then
     # Iterate through all destinations
    for dest in "${destinations[@]}"; do
      mozilla_dir=${destination_paths[$dest]}
      apply_to_mozilla_dir "$mozilla_dir" "$dest"
    done
  else
    # Handle single selection
    selected_dest=${destinations[$((selection - 1))]}
    selected_mozilla_dir=${destination_paths[$selected_dest]}
    apply_to_mozilla_dir "$selected_mozilla_dir" "$selected_dest"
  fi
  exit 0
}

reset_action() {
  detect_applied_mozilla_dirs
  list_mozilla_dirs
  select_mozilla_dir

  if [ "$selection" = 'a' ]; then
    # Iterate through all destinations
    for dest in "${destinations[@]}"; do
      mozilla_dir=${destination_paths[$dest]}
			reset_mozilla_dir "$mozilla_dir" "$dest"
		done
    else
     # Handle single selection
      selected_dest=${destinations[$((selection - 1))]}
			selected_mozilla_dir=${destination_paths[$selected_dest]}
			reset_mozilla_dir "$selected_mozilla_dir" "$selected_dest"
  fi
  exit 0
}

if [ "$#" -eq 0 ]; then
  show_help
  exit 0
fi

action=$1
shift

# Parse CLI args/actions
while [[ "$#" -gt 0 ]]; do
  case $1 in
    -h|--help)
      show_help
      exit 0
      ;;
    -v|--verbose)
      verbose=1
      shift
      ;;
	  --installed)
	    installed=1
		  shift
		  ;;
    *)
    echo "Unexpected argument: $1"
    show_help
    exit 1
    ;;
  esac
done

case $action in
  apply)
    apply_action
    ;;
  list)
    list_action
		;;
  reset)
    reset_action
    ;;
  *)
  echo "Unknown action: $action"
  show_help
  exit 1
   ;;
esac
