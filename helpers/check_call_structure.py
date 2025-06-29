# coding=utf8
# the above tag defines encoding for this document and is for Python 2.x compatibility

import re

regex = r"(?:static\svoid\s\w+\()(.+?(?=\s))"

with open("/mnt/glorbus/prog/glReplay/apitrace/ech/glretrace_gl.cpp") as plik:
    test_str = plik.read()
matches = re.finditer(regex, test_str, re.MULTILINE)

for matchNum, match in enumerate(matches, start=1):
    
    print ("{match}".format(matchNum = matchNum, start = match.start(), end = match.end(), match = match.group().split('(')[0].replace('static void', '')))
    
    #for groupNum in range(0, len(match.groups())):
        #groupNum = groupNum + 1
        
        #print ("Group {groupNum} found at {start}-{end}: {group}".format(groupNum = groupNum, start = match.start(groupNum), end = match.end(groupNum), group = match.group(groupNum)))

# Note: for Python 2.7 compatibility, use ur"" to prefix the regex and u"" to prefix the test string and substitution.
