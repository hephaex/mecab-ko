#/bin/bash

date
MECAB_EXEC_PATH=/usr/local/libexec/mecab
DICT_INDEX=$MECAB_EXEC_PATH/mecab-dict-index
DICT_GEN=$MECAB_EXEC_PATH/mecab-dict-gen
COST_TRAIN=$MECAB_EXEC_PATH/mecab-cost-train

# clear final directory
rm -f ../final/*.csv ../final/*.def ../final/*.def.org ../final/*.bin ../final/*.dic ../final/dicrc 
pushd ../final
./clean
popd

# clear seed directory
rm -f *.dic *.bin model model.txt
$DICT_INDEX -p -d . -c UTF-8 -t UTF-8 -f UTF-8

model_file="model"
corpus_file="corpus/eunjeon_corpus.txt"
cpu_count=`grep -c '^processor' /proc/cpuinfo`
$COST_TRAIN -p ${cpu_count} -c 1.0  ${corpus_file} ${model_file}


cp pos-id.def ../final/.
$DICT_GEN -o ../final -m $model_file

rm -rf ../final.org
cp -R ../final ../final.org

./change_word_cost_ex.sh
./change_word_cost.sh
./change_connection_cost.sh

pushd ../final
./configure; make
popd
date
