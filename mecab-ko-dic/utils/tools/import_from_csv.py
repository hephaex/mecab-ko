#!/usr/bin/python3
# -*-coding:utf-8-*-

# 임시로 사용. 코드 수정 필요

import csv
import getpass
import os
import sys

from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

from dictionary.lexicon import Lexicon

script_path = os.path.dirname(os.path.realpath(__file__))
sys.path.append(script_path + '/..')

table_name = 'lexicon_2_0'

engine = None
db_session = None
base_dir = os.path.dirname(os.path.realpath(__file__))


def connect_db(db_type, host, database, user, password):
    global engine
    global db_session
    if db_type == 'mysql':
        url = 'mysql+mysqlconnector://%s:%s@%s/%s' % \
              (user, password, host, database)
        print('connect to ' + url)
        engine = create_engine(url)
        Session = sessionmaker(bind=engine)
        db_session = Session()
        return db_session
    else:
        raise RuntimeError('Invalid db type: ' + type)


def get_lexicons_from_csv(csv_file_name):
    output = []
    with open(csv_file_name, 'r') as csv_file:
        dictionary_reader = csv.reader(csv_file)
        for row in dictionary_reader:
            if len(row) == 0:
                break
            # 가능역,0,0,0,NNP,지명,,가능역,Compound,*,*,+역,가능/NNG/*+역/NNG/*,Place-station
            if len(row) == 14:
                class_name = row[13]
            else:
                class_name = None
            lexicon = Lexicon(surface=row[0],
                              pos=row[4],
                              read=row[7],
                              semantic_class=row[5],
                              type_name=row[8],
                              start_pos=row[9],
                              end_pos=row[10],
                              expression=row[12],
                              class_name=class_name,
                              is_available=1,
                              is_inspected=1)
            #print(lexicon)
            output.append(lexicon)
    return output


def add_to_db(lexicons):
    try:
        for l in lexicons:
            if not is_exist(l):
                db_session.add(l)
            else:
                print(l)
        db_session.commit()
    except:
        db_session.rollback()
        raise


def is_exist(lexicon):
    sql = """
    SELECT surface
    FROM lexicon_2_0
    WHERE surface=:surface AND pos=:pos AND semantic_class=:semantic_class AND
        is_available = 1
    """
    row = db_session.execute(
        sql,
        {'surface': lexicon.surface,
         'pos': lexicon.pos,
         'semantic_class': lexicon.semantic_class}).first()
    if row is None:
        return False
    else:
        return True


def main():
    host = input('Host: ')
    user = input('User: ')
    password = getpass.getpass()
    connect_db('mysql', host, 'eunjeon', user, password)
    csv_file_name = '/home/bibreen/PycharmProjects/make-dictionary/data/dictionary/place-station/multi_compound_station.csv'
    lexicons = get_lexicons_from_csv(csv_file_name)
    add_to_db(lexicons)


if __name__ == '__main__':
    main()
