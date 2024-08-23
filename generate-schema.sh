#!/bin/bash

echo "generate schema!"
if ! command -v flatc &> /dev/null
then
    echo "flatc could not be found!"
    exit 0
fi

flatc --version
flatc --rust -o src/generated game_schema.fbs