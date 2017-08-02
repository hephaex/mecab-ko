#!/bin/bash

# 샘플 문장을 이용한 사전 테스트 스크립트

DIC_PATH=../../final
INPUT=input.txt
OUTPUT=output.txt
EXPECTED_OUTPUT=expected_output.txt

rm -f output.txt
while read line; do
    echo $line | mecab -d $DIC_PATH
done < $INPUT >> $OUTPUT

result=$(diff output.txt expected_output.txt)
if [ $? == 0 ]; then
  echo -e "\e[1;32m[ SUCCESS ]\e[0m"
else
  echo -e "\e[1;31m[ FAILURE ]\e[0m"
  echo "$result"
fi
