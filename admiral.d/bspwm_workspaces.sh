#!/usr/bin/env bash

if ! type bspc &> /dev/null; then
	echo "bspc not found"
	exit 1
fi

while read -r line; do
	echo -n "%{A4:bspc desktop -f prev:}%{A5:bspc desktop -f next:}"
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
						echo -n " %{A:bspc desktop -f $name:}"
						echo -n "f"
						echo -n "%{A} "
						;;
					F*)
						# focused free desktop
						echo -n " %{A:bspc desktop -f $name:}"
						echo -n "F"
						echo -n "%{A} "
						;;
					o*)
						# occupied desktop
						echo -n " %{A:bspc desktop -f $name:}"
						echo -n "o"
						echo -n "%{A} "
						;;
					O*)
						# focused occupied desktop
						echo -n " %{A:bspc desktop -f $name:}"
						echo -n "O"
						echo -n "%{A} "
						;;
					u*)
						# urgent desktop
						echo -n " %{A:bspc desktop -f $name:}"
						echo -n "u"
						echo -n "%{A} "
						;;
					U*)
						# focused urgent desktop
						echo -n " %{A:bspc desktop -f $name:}"
						echo -n "U"
						echo -n "%{A} "
						;;
				esac
				shift
			done
	esac
	echo -n "%{A}%{A}"
	echo
done < <(bspc subscribe report)
