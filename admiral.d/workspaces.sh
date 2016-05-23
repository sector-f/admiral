#!/usr/bin/env bash

if ! type bspc &> /dev/null; then
	echo "bspc not found"
	exit 1
fi

while read -r line; do
	case $line in
		W*)
			IFS=':'
			set -- ${line#?}
			while [ $# -gt 0 ]; do
				item="$1"
				name="${item#?}"
				case $item in
					f*)
						# free desktop
						echo -n "f"
						;;
					F*)
						# focused free desktop
						echo -n "F"
						;;
					o*)
						# occupied desktop
						echo -n "o"
						;;
					O*)
						# focused occupied desktop
						echo -n "O"
						;;
					u*)
						# urgent desktop
						echo -n "u"
						;;
					U*)
						# focused urgent desktop
						echo -n "U"
						;;
				esac
				shift
			done
	esac
	echo
done < <(bspc subscribe report)
