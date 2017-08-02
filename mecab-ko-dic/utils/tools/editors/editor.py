#!/usr/bin/python3
# -*-coding:utf-8-*-

"""
DB에 오류가 있을 때, 사용되는 스크립트의 샘플
"""
import getpass
import os
import sys
import argparse
import pprint

from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

from dictionary.lexicon import Lexicon


class Editor(object):
    def __init__(self, db_type, host, database, user, password):
        self.db_type = db_type
        self.host = host
        self.database = database
        self.user = user
        self.password = password

        self.db_session = None

    def get_session(self):
        if self.db_session is None:
            self.__connect_db()
        return self.db_session

    def close(self):
        if self.db_session is not None:
            self.db_session.close()
            self.db_session = None

    def __connect_db(self):
        if self.db_type == 'mysql':
            url = 'mysql+pymysql://%s:%s@%s/%s?charset=utf8' % \
                  (self.user, self.password, self.host, self.database)
            print('connect to ' + url)
            engine = create_engine(url)
            Session = sessionmaker(bind=engine)

            self.db_session = Session()
        else:
            raise RuntimeError('Invalid db type: ' + type)

    def edit(self, is_apply):
        if is_apply is False:
            print("##################################################")
            print("This is TEST MODE. it's not aplplied to database.")
            print("for applying to database, use '-a' parameter.")
            print("##################################################")
        lexicons = self.get_lexicons()

        for lexicon in lexicons:
            new_lexicons = self.modify(lexicon)
            if isinstance(lexicon, Lexicon):
                print('### MODIFY #############################')
                pprint.pprint(lexicon.__dict__)
            for new_lexicon in new_lexicons:
                print('### ADD #############################')
                pprint.pprint(new_lexicon.__dict__)
                self.get_session().add(new_lexicon)

        if is_apply:
            print('writing to database.')
            self.get_session().commit()

        self.close()
        print('total count: %s' % len(lexicons))


    # @virtual
    def get_lexicons(self):
        # TODO: write select query.
        # return 'query'
        return []

    # @virtual
    def modify(self, lexicon):
        # TODO: modify lexicon
        return []
