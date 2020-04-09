import sys
import os.path
import cookielib
import urllib2
import re
from time import sleep
import os
import urllib
import math
from optparse import OptionParser

primenet_baseurl = "https://www.mersenne.org/"
gpu72_baseurl = "https://www.gpu72.com/"

parser = OptionParser()

parser.add_option("-u", "--username", dest="username", help="Primenet user name")
parser.add_option("-p", "--password", dest="password", help="Primenet password")

(options, args) = parser.parse_args()

login_data = {"user_login": options.username,
              "user_password": options.password,
              }

# This makes a POST instead of GET
data = urllib.urlencode(login_data)
print("data: " + data)

test_string = """Test=7A30B8B6C0FC79C534A271D9561F7DCC,89459323,76,1
DoubleCheck=92458E009609BD9E10577F83C2E9639C,50549549,73,1
PRP=BC914675C81023F252E92CF034BEFF6C,1,2,96364649,-1,76,0
PRP=51D650F0A3566D6C256B1679C178163E,1,2,81348457,-1,75,0,3,1"""

def greplike(pattern, l):
    output = []
    for line in l:
        s = re.search(r"(" + pattern + ")$", line)
        if s:
            output.append(s.groups()[0])
    return output

print(greplike(r"(DoubleCheck|Test|PRP)\s*=\s*([0-9A-F]){32}(,[0-9]+){3}.*", test_string.split("\n")))