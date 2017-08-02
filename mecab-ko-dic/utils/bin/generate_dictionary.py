#!/usr/bin/python3
# -*-coding:utf-8-*-
import csv
import getpass
import os
import sys

from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

script_path = os.path.dirname(os.path.realpath(__file__))
sys.path.append(script_path + '/..')
from dictionary.utils import get_end_with_jongsung

engine = None
db_session = None
base_dir = os.path.dirname(os.path.realpath(__file__))
target_dir = base_dir + '/../../seed'


def connect_db(db_type, host, database, user, password):
    global engine
    global db_session
    if db_type == 'mysql':
        # url = 'mysql+mysqlconnector://%s:%s@%s/%s?charset=utf8' % \
        url = 'mysql+pymysql://%s:%s@%s/%s?charset=utf8' % \
              (user, password, host, database)
        print('connect to ' + url)
        engine = create_engine(url)
        Session = sessionmaker(bind=engine)
        db_session = Session()
        return db_session
    else:
        raise RuntimeError('Invalid db type: ' + type)


def generate_dictionary_csv_files():
    names = get_pos_names()
    for name in names:
        rows = select_pos_rows(name)
        create_dictionary_csv_file(rows, name)

    names = get_class_names()
    for name in names:
        rows = select_class_rows(name)
        create_dictionary_csv_file(rows, name)


def get_pos_names():
    output = []
    sql = """
    SELECT distinct(pos) FROM lexicon_2_0 WHERE class IS NULL
    """
    rows = db_session.execute(sql).fetchall()
    for row in rows:
        pos = row['pos']
        output.append(pos)
    return output


def get_class_names():
    output = []
    sql = """
    SELECT distinct(class) FROM lexicon_2_0 WHERE class IS NOT NULL
    """
    rows = db_session.execute(sql).fetchall()
    for row in rows:
        pos = row['class']
        output.append(pos)
    return output


def select_pos_rows(pos):
    sql = """
    SELECT  surface, pos, semantic_class, `read`, `type`, start_pos, end_pos,
        expression
    FROM lexicon_2_0
    WHERE pos=:pos AND class IS NULL AND is_available = 1
    ORDER BY surface ASC
    """
    rows = db_session.execute(sql, {'pos': pos}).fetchall()
    return rows


def select_class_rows(class_name):
    sql = """
    SELECT  surface, pos, semantic_class, `read`, `type`, start_pos, end_pos,
        expression
    FROM lexicon_2_0
    WHERE class=:cls AND is_available = 1
    ORDER BY surface ASC
    """
    rows = db_session.execute(sql, {'cls': class_name}).fetchall()
    return rows


def create_dictionary_csv_file(rows, file_name):
    file_path = target_dir + '/' + file_name + '.csv'
    with open(file_path, 'w') as csv_file:
        writer = csv.writer(csv_file, 'unix', quoting=csv.QUOTE_MINIMAL)
        for row in rows:
            write_dic(writer, row)


def write_dic(writer, row):
    writer.writerow([row['surface'],
                     '0', '0', '0',
                     row['pos'],
                     row['semantic_class'],
                     get_end_with_jongsung(row['read']),
                     row['read'],
                     row['type'],
                     row['start_pos'],
                     row['end_pos'],
                     row['expression']])


def main():
    host = input('Host: ')
    user = input('User: ')
    password = getpass.getpass()
    connect_db('mysql', host, 'eunjeon', user, password)
    generate_dictionary_csv_files()


if __name__ == '__main__':
    main()
