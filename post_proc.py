import json;

errors = []
with open('errors.json') as json_file:
    scode = []
    with open('helpers/try.rs', 'r') as helpers_file:
        scode = helpers_file.readlines()
    for line in json_file:
        errors.append(json.loads(line))
    for line in errors:
        try:
            if(line["message"]["level"] == "error"):
                if (line["message"]["spans"][0]["is_primary"]):
                    if (line["message"]["spans"][0]["text"][0]["text"].find("unsafe { gl::") != -1):
                        pos = line["message"]["spans"][0]["line_end"]-1
                        scode[pos] = "//not found in gl!" + scode[pos]
                else:
                    if (line["message"]["spans"][1]["text"][0]["text"].find("unsafe { gl::") != -1):
                        pos = line["message"]["spans"][1]["line_end"]-1
                        scode[pos] = "//not found in gl!" + scode[pos] 

            # if (line["reason"] != "compiler-artifact" and line["reason"].find('compiler') != -1):
            #     if(line["message"]["spans"][0]["label"] == "not found in `gl`"):
            #         pos = line["message"]["spans"][0]["line_start"]-1
            #         scode[pos] = "//not found in gl!" + scode[pos]
            #         if(scode[pos].find("let _result") != -1):
            #             scode[pos] = scode[pos] + "\nlet _result = 0;"
            #     if(line["message"]["spans"][0]["label"] == "not found in this scope" and line["message"]["spans"][0]["text"][0]["text"].find("pub") == -1 and line["message"]["spans"][0]["text"][0]["text"].find("if") == -1 and line["message"]["spans"][0]["text"][0]["text"].find("let") == -1 and line["message"]["spans"][0]["text"][0]["text"].find("for") == -1):
            #         pos = line["message"]["spans"][0]["line_end"]-1
            #         scode[pos] = "//not found in scope!" + scode[pos]
            #     try:
            #         print(line["message"]["spans"]) if (line["message"]["spans"][0]["label"]!= "not found in `gl`" and line["message"]["spans"][0]["label"].find("use of unresolved module or unlinked crate `") == -1) else 0
            #         if(line["message"]["spans"][1]["label"] == "similarly named function `ColorP3ui` defined here"):
            #             pos = line["message"]["spans"][0]["line_start"]-1
            #             scode[pos] = "//not found in gl!" + scode[pos]
            #     except:
            #         pass
        except Exception as e:
            print(e)
    with open('helpers/try.rs', 'w') as helpers_file:
        helpers_file.writelines(scode)