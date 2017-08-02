__author__ = 'budditao'

import getpass
import os
import sys
import argparse
import pprint

script_path = os.path.dirname(os.path.realpath(__file__))
sys.path.append(script_path + '/..')
from tools.editors.xsn_editor import XsnEditor
from tools.editors.compound_editor import CompoundEditor
from tools.editors.inflect_editor import InflectEditor
from tools.editors.street_insertor import StreetNameEditor

def main():
    parser = argparse.ArgumentParser(prog='PROG')
    parser.add_argument('--host', help='host')
    parser.add_argument('-u', '--user', help='user')
    parser.add_argument('-p', '--password', default=None, help='password')
    parser.add_argument('-a', '--apply', default=False, action='store_true',
                        help='apply to database')
    args = parser.parse_args()
    if args.password is None:
        args.password = getpass.getpass()

    inflect_editor = InflectEditor('mysql', args.host, 'eunjeon', args.user, args.password)
    inflect_editor.edit(args.apply)
    # compound_editor = CompoundEditor('mysql', args.host, 'eunjeon', args.user, args.password)
    # compound_editor.edit(args.apply)
    # editor = XsnEditor('mysql', args.host, 'eunjeon', args.user, args.password)
    # editor.edit(args.apply)
    #street_name_editor = StreetNameEditor('mysql', args.host, 'eunjeon', args.user, args.password)
    #street_name_editor.edit(args.apply)


if __name__ == '__main__':
    main()
