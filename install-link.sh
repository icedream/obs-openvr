#!/bin/bash

set -e

target_dir="$(realpath "$(dirname "$0")")/target"

plugin_dir=~/.config/obs-studio/plugins
plugin_name=obs-openvr
force=0
while getopts ':d:p:f' opt; do
	case ${opt} in
		d)
			plugin_dir="$OPTARG"
			;;
		p)
			plugin_name="$OPTARG"
			;;
		f)
			force=1
			;;
		\?)
			echo "Unknown argument: -$OPTARG" >&2
			exit 1
			;;
		:)
			echo "Invalid argument: -$OPTARG requires an argument" >&2
			exit 1
			;;
	esac
done
shift $((OPTIND - 1))

profile="${1:-debug}"

ensure_dir() {
	local d
	for d; do
		[ -d "$d" ] || mkdir -p "$d"
	done
}

bin_dir="$plugin_dir/$plugin_name/bin/64bit"

ensure_dir "$bin_dir"

plugin_file="$(printf '%s/lib%s.so' "$bin_dir" "$plugin_name")"

ln -f -s "$target_dir/$profile/libobs_openvr.so" "$plugin_file"
file "$plugin_file"
