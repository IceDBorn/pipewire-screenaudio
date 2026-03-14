#!/usr/bin/env bash
set -e

projectRoot="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P )"
jsonTemplate="$projectRoot/native/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json"

function prompt_question() {
	local prompt="$1"
	while true; do
		read -p "$prompt [Y/n]: " yn

		[ -z "$yn" ] && return 0 # Default to yes on empty input

		case $yn in
			[Yy]*) return 0 ;;
			[Nn]*) return 1 ;;
			*) echo "Please answer yes (y) or no (n)." ;;
		esac
	done
}

function prompt_browser_type() {
	while true; do
		read -p "Select browser type (1 = Firefox-based, 2 = Chromium-based): " browser_num

		case $browser_num in
			1) BROWSER="firefox"; return ;;
			2) BROWSER="chromium"; return ;;
			*) echo "Invalid selection. Please enter 1 for Firefox-based or 2 for Chromium-based." ;;
		esac
	done
}

function prompt_native_messaging_hosts_path() {
	local dir

	while true; do
		read -p "Directory path: " dir
		if [[ -z "$dir" ]]; then
			echo "Directory path cannot be empty. Please try again."
			continue
		fi

		dir=$(eval echo "$dir") # Replace ~/ with absolute path
		if [[ ! -d "$dir" ]]; then
			echo "Directory \"$dir\" does not exist."

			while true; do
				read -p "Do you want to create it? [y/N]: " create_dir
				case $create_dir in
					[Yy]*)
						mkdir -p "$dir" && echo "Created directory: $dir"
						break
						;;
					[Nn]*|"")
						echo -e "\nPlease provide a valid directory path or create it."
						continue 2
						;;
					*)
						echo "Please answer yes (y) or no (n)."
						;;
				esac
			done
		fi

		DIR="$dir"
		return
	done
}

function prompt_chromium_extension_id() {
	local default_id="dfoogepphhgaoofihnedpnmjgjoglglm"
	while true; do
		read -p "Provide the extension ID (default: $default_id). Press enter to use default: " extid
		if [[ -z "$extid" ]]; then
			echo "$default_id"
			return
		else
			echo "$extid"
			return
		fi
	done
}

# Build steps
if prompt_question "Do you want to build the extension?"; then
	( cd "$projectRoot/extension/react" && yarn install && npx vite build )
	echo -e
else
	echo "Skipping extension build."
fi

if prompt_question "Do you want to build the native connector?"; then
	( cd "$projectRoot/native/connector-rs" && cargo build )
	echo -e
else
	echo "Skipping native connector build."
fi

# Browser type selection
prompt_browser_type

# Native messaging hosts path selection
echo -e "\nProvide the browser native messaging hosts directory path. Common paths include:"
if [[ "$BROWSER" == "firefox" ]]; then
	echo "~/.mozilla/native-messaging-hosts, ~/.config/mozilla/firefox/native-messaging-hosts, ~/.librewolf/native-messaging-hosts"
	prompt_native_messaging_hosts_path
	allowed_field="allowed_extensions"
	allowed_value="pipewire-screenaudio@icenjim"
elif [[ "$BROWSER" == "chromium" ]]; then
	echo "~/.config/net.imput.helium/NativeMessagingHosts, ~/.config/chromium/NativeMessagingHosts, ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts, ~/.config/google-chrome/NativeMessagingHosts"
	prompt_native_messaging_hosts_path
	extid=$(prompt_chromium_extension_id)
	allowed_field="allowed_origins"
	allowed_value="chrome-extension://$extid/"
fi

# Prepare and install json to native messaging hosts path
connector_path="$projectRoot/native/connector-rs/target/debug/connector-rs"
output_json="$DIR/com.icedborn.pipewirescreenaudioconnector.json"
tmp_json=$(mktemp)

cp "$jsonTemplate" "$tmp_json"
sed -i "s|CONNECTOR_BINARY_PATH|$connector_path|g" "$tmp_json"
sed -i -E "s|\"allowed_\*\"[[:space:]]*:[[:space:]]*\[[[:space:]]*\"EXTENSION_ID\"[[:space:]]*\]|\"$allowed_field\": [\"$allowed_value\"]|g" "$tmp_json"

if cp "$tmp_json" "$output_json"; then
	echo "Installed native messaging hosts manifest json file to $output_json"
else
	echo "Failed to write manifest. Please check permissions and try again."
	exit 1
fi

rm "$tmp_json"
