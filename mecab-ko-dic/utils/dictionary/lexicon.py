# -*-coding:utf-8-*-
import re

from datetime import datetime

from sqlalchemy import Column,Integer, String, DateTime, Sequence, Index, \
    UniqueConstraint
from sqlalchemy.ext.declarative import declarative_base

Base = declarative_base()


class Lexicon(Base):

    __tablename__ = 'lexicon_2_0'

    id = Column(Integer, Sequence('user_id_seq'), primary_key=True)
    surface = Column(String(64), nullable=False, index=True)
    pos = Column(String(64), nullable=False, index=True)
    semantic_class = Column(String(16), default="*", nullable=False)
    read = Column(String(64), nullable=False)
    type_name = Column(String(16), name='type', default="*", nullable=False)
    start_pos = Column(String(16), default="*", nullable=False)
    end_pos = Column(String(16), default="*", nullable=False)
    expression = Column(String(128), default="*", nullable=False)
    class_name = Column(String(64), name='class', index=True)
    is_available = Column(Integer, nullable=False, index=True)
    is_inspected = Column(Integer, nullable=False, index=True)
    last_modified = Column(DateTime,
                           default=datetime.now(),
                           nullable=False,
                           index=True)
    comment = Column(String(256))
    __table_args__ = (UniqueConstraint('surface',
                                       'pos',
                                       'semantic_class',
                                       name='idx_surface_pos_semantic_class'),)
    def __init__(self,
                 surface,
                 pos,
                 read,
                 semantic_class='*',
                 type_name='*',
                 start_pos='*',
                 end_pos='*',
                 expression='*',
                 class_name=None,
                 is_available=1,
                 is_inspected=0,
                 last_modified=datetime.now()):
        self.surface = surface
        self.pos = pos
        self.semantic_class = semantic_class
        self.read = read
        self.type_name = type_name
        self.start_pos = start_pos
        self.end_pos = end_pos
        self.expression = expression
        self.class_name = class_name if class_name else '*'
        self.is_available = is_available if is_available else 1
        self.is_inspected = is_inspected if is_inspected else 0


    def __repr__(self):
        return '<' + ','.join((self.surface,
                               self.pos,
                               self.semantic_class,
                               self.read,
                               self.type_name,
                               self.start_pos,
                               self.end_pos,
                               self.expression,
                               self.class_name,
                               str(self.is_available),
                               str(self.is_inspected))) + '>'
