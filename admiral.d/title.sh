#!/usr/bin/env bash

if ! type xtitle &> /dev/null; then
	echo "xtitle not found"
	exit 1
fi

while IFS= read -r title; do
	echo "%{c}$title"
done < <(xtitle -s -t 100)
