#!/bin/bash

# 각 단어의 출현 비용을 변경하는 스크립트

DIC_PATH='../final'
CONF_FILE='change_word_cost.txt'

function get_dicts()
{
    local surface=$1
    local tag=$2
    local semantic_class=$3
    grep -l -E "^$surface,[0-9]+,[0-9]+,[-0-9]+,$tag,$semantic_class" $DIC_PATH/*.csv
}

function change_word_cost()
{
    local dict=$1
    local surface=$2
    local tag=$3
    local semantic_class=$4
    local new_cost=$5
    grep -E "^$surface,[0-9]+,[0-9]+,[-0-9]+,$tag,$semantic_class" $dict
    sed -i -re "s/($surface,[0-9]+,[0-9]+,)([-0-9]+)(,$tag,$semantic_class,)/\\1$new_cost\\3/g" $dict
}

## MAIN
while read line; do
    if [ "${line:0:1}" == "#" ]; then
        continue
    fi
    surface=$(echo $line | cut -d ',' -f 1)
    tag=$(echo $line | cut -d ',' -f 2)
    semantic_class=$(echo $line | cut -d ',' -f 3)
    new_cost=$(echo $line | cut -d ',' -f 4)

    dict_list=$(get_dicts "$surface" "$tag" "$semantic_class")
    for each in $dict_list; do
        change_word_cost $each $surface $tag $semantic_class $new_cost
        echo "word cost change... $each:$surface,$tag,$semantic_class -> $new_cost"
    done
done < $CONF_FILE
