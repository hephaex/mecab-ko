#!/usr/bin/python3
# -*-coding:utf-8-*-
import csv
import os
import sys

from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

script_path = os.path.dirname(os.path.realpath(__file__))
sys.path.append(script_path + '/..')

from dictionary.lexicon import Lexicon

engine = None
db_session = None
base_dir = os.path.dirname(os.path.realpath(__file__))
seed_dir = base_dir + '/../../seed'


def connect_db():
    global engine
    global db_session
    url = 'sqlite:///eunjeon.db'
    print('connect to ' + url)
    engine = create_engine(url)
    Session = sessionmaker(bind=engine)
    db_session = Session()
    return db_session


def create_table(reset=True):
    table = Lexicon.__table__
    if table.exists(bind=engine):
        if reset is True:
            table.drop(bind=engine)
        else:
            raise Exception('exists table.')
    table.create(bind=engine, checkfirst=True)


def insert_morphemes_to_db(dir, file_name):
    class_name = file_name[0:-4]
    if class_name.isupper() or len(class_name) > 1:
        class_name = None

    with open(seed_dir + '/' + file_name, 'r') as csv_file:
        reader = csv.reader(csv_file)
        is_inspected = 1
        if class_name == 'Wikipedia':
            is_inspected = 0
        for row in reader:
            morph = Lexicon(surface=row[0],
                             pos=row[4],
                             semantic_class=row[5],
                             read=row[7],
                             type_name=row[8],
                             start_pos=row[9],
                             end_pos=row[10],
                             compound_expression=row[11],
                             index_expression=row[12],
                             class_name=class_name,
                             is_available=1,
                             is_inspected=is_inspected)
            db_session.add(morph)
        db_session.commit()


def main():
    connect_db()
    create_table()
    file_names = os.listdir(seed_dir)
    for file_name in file_names:
        if file_name[-4:] != '.csv':
            continue
        insert_morphemes_to_db(dir, file_name)
        print('Insert to db: ' + file_name)


if __name__ == '__main__':
    main()