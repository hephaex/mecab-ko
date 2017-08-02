#!/usr/bin/python3
# -*-coding:utf-8-*-
"""
DB 데이터 체크 프로그램
"""
import getpass
import os
import sys

from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

script_path = os.path.dirname(os.path.realpath(__file__))
sys.path.append(script_path + '/..')

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


def edit():
    global db_session
    sql = """
    select * from lexicon_new where `type`='Preanalysis' or `type`='Compound';
    """
    rows = db_session.execute(sql).fetchall()
    for row in rows:
        id = row[0]
        index_expr = row[9]
        # print(row)
        # index expression check
        exprs = index_expr.split('+')
        for expr in exprs:
            try:
                items = expr.split('/')
                semantic_class = items[2]
                position_incr = items[3]
                position_length = items[4]
                if len(items) != 5:
                    print('1.ERROR:', id, index_expr)
                if not represents_int(position_incr) or \
                        not represents_int(position_length):
                    print('2.ERROR:', id, index_expr)
                if not (semantic_class == '*' or semantic_class == '인명' or
                        semantic_class == '지명'):
                    print('3.ERROR:', id, index_expr)
            except:
                print('Exception: ', row)


def represents_int(s):
    try:
        int(s)
        return True
    except ValueError:
        return False


def main():
    host = input('Host: ')
    user = input('User: ')
    password = getpass.getpass()
    connect_db('mysql', host, 'eunjeon', user, password)
    edit()


if __name__ == '__main__':
    main()
