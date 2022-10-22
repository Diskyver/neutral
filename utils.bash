#!/usr/bin/env bash

function crate_version {
    local in_package=0

    for line in $(cat ./Cargo.toml); do
        if [[ $in_package -eq 3 ]]; then
            echo "${line}" | tr -d '"'
            return 0 
        fi

        if [[ $in_package -eq 2 ]]; then
            in_package=3 
        fi

        if [[ $in_package -eq 1 ]]; then
            if [[ "${line}" == "version" ]]; then
                in_package=2
            fi
        fi

        if [[ "${line}" == "[package]" ]]; then
            in_package=1
        fi
    done

    return 1
}

function release_version {
    local remote="${1:-origin}"
    local version="$(crate_version)"
    git tag "${version}" && git push "${remote}" "${version}"
}