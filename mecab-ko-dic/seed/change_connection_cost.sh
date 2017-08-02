#!/bin/bash

# matrix.def의 연접 비용을 변경하는 스크립트

DIC_PATH='../final'
MATRIX_DEF=matrix.def
LEFT_ID_DEF=left-id.def
RIGHT_ID_DEF=right-id.def
CONF_FILE=change_connection_cost.txt

function get_left_id()
{
    local tag=$1
    local semantic_class=$2
    local reading=$3

    grep -E -m 1 "^[0-9]+ $tag,$semantic_class,\*,$reading" $DIC_PATH/$LEFT_ID_DEF | cut -d ' ' -f 1
}

function get_right_id()
{
    local tag=$1
    local semantic_class=$2
    local has_last_jongsung=$3
    local reading=$4

    if [ -z "$reading" ]; then
        grep -E -m 1 "^[0-9]+ $tag,$semantic_class,$has_last_jongsung" $DIC_PATH/$RIGHT_ID_DEF | cut -d ' ' -f 1
    else
        grep -E -m 1 "^[0-9]+ $tag,$semantic_class,$has_last_jongsung,$reading" $DIC_PATH/$RIGHT_ID_DEF | cut -d ' ' -f 1
    fi
}

function get_connection_cost()
{
    local left_id=$1
    local right_id=$2

    grep -E -m 1 "^$right_id $left_id " $DIC_PATH/$ORG_MATRIX_DEF | cut -d ' ' -f 3
}

function get_sed_command_for_new_cost()
{
    local left_id=$1
    local right_id=$2
    local new_cost=$3

    local cost=$(get_connection_cost $left_id $right_id)
    echo "s/$right_id $left_id $cost\$/$right_id $left_id $new_cost/g"
}

## MAIN
ORG_MATRIX_DEF=matrix.def.org
if [ ! -e $DIC_PATH/$ORG_MATRIX_DEF ]; then
    cp $DIC_PATH/$MATRIX_DEF $DIC_PATH/$ORG_MATRIX_DEF
fi

sed_patterns=''

while read line; do
    if [ "${line:0:1}" == "#" ]; then
        continue
    fi
    connection=$(echo $line | cut -d ' ' -f 1)
    new_cost=$(echo $line | cut -d ' ' -f 2)

    left=$(echo $connection | cut -d '|' -f 1)
    right=$(echo $connection | cut -d '|' -f 2)

    l_tag=$(echo $left | cut -d ',' -f 1)
    l_semantic_class=$(echo $left | cut -d ',' -f 2)
    l_jongsung=$(echo $left | cut -d ',' -f 3)
    l_pron=$(echo $left | cut -d ',' -f 4)

    r_tag=$(echo $right | cut -d ',' -f 1)
    r_semantic_class=$(echo $right | cut -d ',' -f 2)
    r_pron=$(echo $right | cut -d ',' -f 3)

    right_id=$(get_right_id "$l_tag" "$l_semantic_class" "$l_jongsung" "$l_pron")
    left_id=$(get_left_id "$r_tag" "$r_semantic_class" "$r_pron")
    sed_patterns="$sed_patterns$(get_sed_command_for_new_cost $left_id $right_id $new_cost);"

    cost=$(get_connection_cost $left_id $right_id)
    echo "$left | $right $cost->$new_cost"
done < $CONF_FILE

echo "connection cost change... '$sed_patterns'"
sed "$sed_patterns" $DIC_PATH/$ORG_MATRIX_DEF > $DIC_PATH/$MATRIX_DEF
