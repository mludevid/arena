#!/usr/bin/env python3

import os
import subprocess
import sys

def get_test_files():
    test_files = []
    for (dirpath, dirnames, filenames) in os.walk('tests'):
        for filename in filenames:
            path = os.sep.join([dirpath, filename])
            if path.endswith(".arena"):
                test_files.append(path)
    return test_files

def get_result(path):
    with open(path) as f:
        first_line = f.readline()[:-1]
        result_header = "// Result:"
        if first_line == result_header:
            prefix = "// "
            result_lines = []
            line = f.readline()
            while line.startswith(prefix):
                result_lines.append(line[len(prefix):-1])
                line = f.readline()
            return result_lines

def run(cmd):
    proc = subprocess.Popen(cmd,
        stdout = subprocess.PIPE,
        stderr = subprocess.PIPE,
    )
    stdout, stderr = proc.communicate()
    stdout = stdout.decode('utf-8')
    stderr = stderr.decode('utf-8')
    return proc.returncode, stdout, stderr

def get_execution_result(path):
    ret_code, stdout, stderr = run(["./arena", path])
    if ret_code != 0:
        print("\nCompilation of " + str(path) + " failed. StdErr:\n")
        print(str(stderr) + "\n")
        return
    ret_code, stdout, stderr = run("./out")
    # TODO: When ARENA does always return 0 on successfull run:
    """
    if ret_code != 0:
        print(str(path) + " failed. StdErr:")
        print(stderr)
    """
    return str(stdout)

def perform_test(path):
    res = get_result(path)
    if res is not None:
        out = get_execution_result(path)
        out_lines = out.split("\n")[:-1]
        if out_lines == res:
            print("Passed")
            return True
        else:
            print("Failed: " + path)
            print("Expected:")
            print(res)
            print("Got:")
            print(out_lines)
            return False
    else:
        print("Skiping " + str(path))

if __name__ == '__main__':
    if len(sys.argv) == 1:
        passed = 0
        failed = 0
        print("Testing:")
        for file in get_test_files():
            res = perform_test(file)
            if res == True:
                passed += 1
            if res == False:
                failed += 1
        print("")
        print("Results:")
        print("Passed: " + str(passed))
        print("Failed: " + str(failed))
    elif len(sys.argv) == 2:
        perform_test(sys.argv[1])
    else:
        print("Please only provide one test at a time")

