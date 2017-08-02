#!/bin/bash

if [ -z "`command -v gawk`" ]; then
    AWK=awk
    LENGTH_4=12
    LENGTH_3=9
    LENGTH_2=6
else
    AWK=gawk
    LENGTH_4=4
    LENGTH_3=3
    LENGTH_2=2
fi
TARGET_DIR='../final'

# 인명 cost 재조정
# '이승기 => 이 + 승기'로 오분석 되는 것 방지
for each in `ls $TARGET_DIR/Person*.csv`; do
    echo "change word cost in $each ..."
    cat $each | $AWK -F ',' -v length_3=$LENGTH_3 -v length_2=$LENGTH_2 'BEGIN {
        OFS = ","
    }
    {
        surface = $1
        decr_cost = $4
        if (length(surface) == length_2) {
            # do nothing (추후 필요하면...)
        } else if (length(surface) >= length_3) {
            where = match(surface, "^이")
            if (where && decr_cost > 0) {
                decr_cost = int(decr_cost * 0.4);
            } else {
                decr_cost = int(decr_cost * 0.75);
            }
        }
        print($1,$2,$3,decr_cost,$5,$6,$7,$8,$9,$10,$11,$12)
    }' > $each.tmp
    rm -f $each
    mv $each.tmp $each
done

# 지명 cost 재조정
# '내방 => 내 + 방'으로 오분석 되는 것 방지
for each in `ls $TARGET_DIR/Place*.csv`; do
    echo "change word cost in $each ..."
    cat $each | $AWK -F ',' -v length_3=$LENGTH_3 -v length_2=$LENGTH_2 'BEGIN {
        OFS = ","
    }
    {
        surface = $1
        decr_cost = $4
        if (length(surface) == length_2) {
            where = match(surface, "^내")
            if (where && decr_cost > 0) {
                decr_cost = int(decr_cost * 0.4);
            }
        } else if (length(surface) >= length_3) {
            where = match(surface, "^내")
            if (where && decr_cost > 0) {
                decr_cost = int(decr_cost * 0.4);
            }
        }
        print($1,$2,$3,decr_cost,$5,$6,$7,$8,$9,$10,$11,$12)
    }' > $each.tmp
    rm -f $each
    mv $each.tmp $each
done

# *주의/NNG (ex: 민주주의) cost 재조정
# '자연주의 쇼핑몰' => '자연+주+의(조사) 쇼핑몰'으로 오분석 되는 것 방지
for each in `ls $TARGET_DIR/NNG.csv`; do
    echo "change word cost in $each ..."
    cat $each | $AWK -F ',' -v length_4=$LENGTH_4 'BEGIN {
        OFS = ","
    }
    {
        surface = $1
        decr_cost = $4
        if (length(surface) >= length_4) {
            where = match(surface, "주의$")
            if (where && decr_cost > 0) {
                decr_cost = int(decr_cost * 0.6);
            }
        }
        print($1,$2,$3,decr_cost,$5,$6,$7,$8,$9,$10,$11,$12)
    }' > $each.tmp
    rm -f $each
    mv $each.tmp $each
done
