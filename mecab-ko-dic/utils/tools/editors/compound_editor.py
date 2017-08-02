__author__ = 'budditao'

import pprint
from datetime import datetime
from sqlalchemy import or_

from dictionary.lexicon import Lexicon
from tools.editors.editor import Editor


class CompoundEditor(Editor):

    # @override
    def get_lexicons(self):
        # TODO: write select query
        return self.get_session().query(Lexicon). \
            filter(or_(Lexicon.type_name == 'Compound',
                       Lexicon.type_name == 'Preanalysis')).all()
            #filter(Lexicon.surface=='리차드위드마크').all()
            #filter(Lexicon.surface=='수륙양용기').all()
            #filter(Lexicon.pos == 'NNG').all()

    # @override
    def modify(self, lexicon):
        expression = Expression(lexicon.index_expression)
        new_lexicons = []
        # TODO: rebuild lexicon object
        lexicon.start_pos = expression.get_tokens()[0].pos
        lexicon.end_pos = expression.get_tokens()[-1].pos
        lexicon.compound_expression = expression.__repr__()
        lexicon.last_modified = datetime.now()
        #lexicon.is_available = '0'
        return new_lexicons

    def make_new_index_expression(self, old_expression):
        result = []
        expression = old_expression.split('+')
        for token in expression :
            token_part = token.split('/')
            if token_part[3] == '1':
                result.append(token_part[0])
                result.append(token_part[1])
                result.append(token_part[2])
        return '/'.join(result)

class Token:
    def __init__(self, surface, pos, semantic_class):
        self.surface = surface
        self.pos = pos
        self.semantic_class = semantic_class

    def __repr__(self):
        return '%s/%s/%s' % (self.surface, self.pos, self.semantic_class)

class Expression(object):
    def __init__(self, expression_str):
        self.expression_str = expression_str
        self.parsed_expression = None
        self._parse()

    def _parse(self):
        result = []
        expression = self.expression_str.split('+')
        for token in expression:
            token_part = token.split('/')
            if token_part[3] == '1':
                new_token = Token(token_part[0], token_part[1], token_part[2])
                result.append(new_token)
        self.parsed_expression = result

    def __repr__(self):
        result = []
        for token in self.parsed_expression:
            result.append(token.__repr__())
        return '+'.join(result)

    def get_tokens(self):
        return self.parsed_expression

