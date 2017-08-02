#!/usr/bin/python3
# -*-coding:utf-8-*-


def get_end_with_jongsung(s):
    if not is_hangul(s):
        return '*'
    if end_with_jongsung(s):
        return 'T'
    else:
        return 'F'


def is_hangul(s):
    for c in s:
        uni = ord(c)
        if not (0x0AC00 <= uni <= 0xD7A3 or
                0x1100 <= uni <= 0x11FF or
                0x3130 <= uni <= 0x318F):
            return False
    return True


def end_with_jongsung(s):
    if not is_hangul(s):
        return False
    last_char = s[-1]
    uni = ord(last_char)
    if 44032 <= uni <= 55203 and ((uni - 44032) % 28 > 0):
        return True
    else:
        return False
