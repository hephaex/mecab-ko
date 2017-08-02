__author__ = 'budditao'

import os
from datetime import datetime

from dictionary.lexicon import Lexicon
from tools.editors.editor import Editor


class StreetNameEditor(Editor):
    def get_lexicons(self):
        result = []
        file_path = os.path.dirname(__file__)+'/street_address_dic/street'
        with open(file_path, mode='r') as name_file:
            for line in name_file:
                result.append(line.strip())
        return result

    def modify(self, lexicon):
        # TODO: build lexicons... and return it.
        print(lexicon)
        return []



