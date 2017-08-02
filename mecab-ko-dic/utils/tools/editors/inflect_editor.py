__author__ = 'budditao'

import pprint
from datetime import datetime
from sqlalchemy import or_, and_

from dictionary.lexicon import Lexicon
from tools.editors.editor import Editor


class InflectEditor(Editor):

    # @override
    def get_lexicons(self):
        # TODO: write select query
        return self.get_session().query(Lexicon). \
            filter(Lexicon.type_name == 'Inflect').all()

    # @override
    def modify(self, lexicon):
        expression = Expression(lexicon.compound_expression)
        for token in expression.get_tokens():
            if token.pos == 'NNP':
                place_lexicon = self.get_session().query(Lexicon). \
                        filter(and_(Lexicon.pos=='NNP',
                                    Lexicon.surface==token.surface,
                                    Lexicon.semantic_class=='지명')).first()
                if place_lexicon is not None:
                    token.semantic_class = '지명'
                    continue

                person_lexicon = self.get_session().query(Lexicon). \
                    filter(and_(Lexicon.pos=='NNP',
                                Lexicon.surface==token.surface,
                                Lexicon.semantic_class=='인명')).first()
                if person_lexicon is not None:
                    token.semantic_class = '인명'
                    continue

        # TODO: rebuild lexicon object
        lexicon.compound_expression = expression.__repr__()
        lexicon.last_modified = datetime.now()
        return []


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
            new_token = Token(token_part[0], token_part[1], '*')
            result.append(new_token)
        self.parsed_expression = result

    def __repr__(self):
        result = []
        for token in self.parsed_expression:
            result.append(token.__repr__())
        return '+'.join(result)

    def get_tokens(self):
        return self.parsed_expression

