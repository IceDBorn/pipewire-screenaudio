#!/usr/bin/env bash
set -e

DEFAULT_CHROMIUM_EXT_ID="cbmjbapailadjabjnjnnbfdimkbdicja"
REQUIRED_BINS_JS=("yarn")
REQUIRED_BINS_RUST=("cargo")

projectRoot="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P )"
jsonTemplate="$projectRoot/native/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json"

function prompt_yn_question() {
	local prompt="$1"
	local default="$2"

	if [[ "$default" == "Y" ]]; then
		promptPostfix="[Y/n]"
		defaultReturn="0"
	else
		promptPostfix="[y/N]"
		defaultReturn="1"
	fi

	while true; do
		read -p "$prompt $promptPostfix: " yn

		[ -z "$yn" ] && return $defaultReturn

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
			*) echo -e "Invalid selection.\n" ;;
		esac
	done
}

function prompt_required_value() {
	local prompt="$1"
	local emptyMessage="$2"

	while true; do
		read -p "$prompt" returnValue

		if [[ -z "$returnValue" ]]; then
			[[ ! -z "$emptyMessage" ]] && echo "$emptyMessage" >/dev/stderr
			continue
		fi

		echo "$returnValue"
		break
	done
}

function prompt_native_messaging_hosts_path() {
	local dir

	while true; do
		dir=$(
			prompt_required_value \
				"Directory path: " \
				"Directory path cannot be empty. Please try again."
		)

		dir=$(eval echo "$dir") # Replace ~/ with absolute path
		if [[ ! -d "$dir" ]]; then
			echo "Directory \"$dir\" does not exist."

			if prompt_yn_question "Do you want to create it?" "N"; then
				mkdir -p "$dir" && echo "Created directory: $dir"
			else
				echo -e "Please provide a valid directory path or create it."
				continue 2
			fi
		fi

		DIR="$dir"
		return
	done
}

function prompt_chromium_extension_id() {
	while true; do
		read -p "Provide the extension ID (default: $DEFAULT_CHROMIUM_EXT_ID). Press enter to use default: " extid
		if [[ -z "$extid" ]]; then
			echo "$DEFAULT_CHROMIUM_EXT_ID"
			return
		else
			echo "$extid"
			return
		fi
	done
}

function check_required_bins() {
	for bin in "${@}"; do
		if command -v "$bin" >/dev/null; then
			continue
		fi

		echo "$bin: command not found" >/dev/stderr
		exit 1
	done
}

if [[ "$DEBUG" != 1 ]] && ! prompt_yn_question "Can you confirm that you are following our instructions in INSTALL.md ?" "N"; then
	echo "Please read the instructions before proceeding." >/dev/stderr
	echo "https://github.com/IceDBorn/pipewire-screenaudio/blob/main/INSTALL.md" >/dev/stderr
	exit 1
fi

# Build React
if prompt_yn_question "Do you want to build the extension?" "N"; then
	check_required_bins "${REQUIRED_BINS_JS[@]}"
	( cd "$projectRoot/extension/react" && yarn install && yarn build || exit 1 ) || exit 1
	echo -e
else
	echo "Skipping extension build."
fi

# Build Rust
while true; do
	if [[ -f "/usr/lib/pipewire-screenaudio/connector/connector-rs" ]] && prompt_yn_question "Do you want to use the installed native connector binary?" "Y" ; then
		CONNECTOR_PATH="/usr/lib/pipewire-screenaudio/connector/connector-rs"
	elif prompt_yn_question "Do you want to build the native connector?" "Y"; then
		check_required_bins "${REQUIRED_BINS_RUST[@]}"
		( cd "$projectRoot/native/connector-rs" && cargo build || exit 1 ) || exit 1
		echo -e
		CONNECTOR_PATH="$projectRoot/native/connector-rs/target/debug/connector-rs"
	else
		echo -e "Native connector is required.\n" >/dev/stderr
		continue
	fi

	break
done

# Browser type selection
prompt_browser_type

# Native messaging hosts path selection
echo -e "Provide the browser native messaging hosts directory path:"
if [[ "$BROWSER" == "firefox" ]]; then
	prompt_native_messaging_hosts_path
	ALLOWED_FIELD="allowed_extensions"
	ALLOWED_VALUE="pipewire-screenaudio@icenjim"
elif [[ "$BROWSER" == "chromium" ]]; then
	prompt_native_messaging_hosts_path
	extid=$(prompt_chromium_extension_id)
	ALLOWED_FIELD="allowed_origins"
	ALLOWED_VALUE="chrome-extension://$extid/"
fi

# Prepare and install json to native messaging hosts path
output_json="$DIR/com.icedborn.pipewirescreenaudioconnector.json"
tmp_json=$(mktemp)

# Replace values in JSON
cp "$jsonTemplate" "$tmp_json"
sed -i "s|CONNECTOR_BINARY_PATH|$CONNECTOR_PATH|g" "$tmp_json"
sed -i "s|ALLOWED_FIELD|$ALLOWED_FIELD|g" "$tmp_json"
sed -i "s|ALLOWED_VALUE|$ALLOWED_VALUE|g" "$tmp_json"

if cp "$tmp_json" "$output_json"; then
	echo "Installed native messaging hosts manifest json file to $output_json"
else
	echo "Failed to write manifest. Please check permissions and try again."
	exit 1
fi

rm "$tmp_json"
