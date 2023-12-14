import os
import random
import shutil
import string

import pexpect
from ivoire import describe

EXEC_PREFIX = "db > "
SQLITE_LITE_EXEC_NAME = "sqlite_lite"
DB_FILENAME = "db_file"


class SqliteLite:
    pass


def get_executable():
    shutil.copy(f'../target/debug/{SQLITE_LITE_EXEC_NAME}', '.')


def delete_old_db():
    if os.path.isfile(DB_FILENAME):
        os.remove(DB_FILENAME)


def run_executable(inputs: list[str]):
    c = pexpect.spawnu(f'./{SQLITE_LITE_EXEC_NAME} {DB_FILENAME}')
    final = ""

    for cmd in inputs:
        c.sendline(cmd)
        final += c.read_nonblocking(50000)

    resp = final.split("\n")

    resp = [r.strip() for r in resp if r]

    return resp


with describe(SqliteLite) as it:
    @it.before
    def before(_test):  # Takes at least 1 parameter, test, which can be used in other `it()`. Kinda like Object.self
        _test.maxDiff = None
        delete_old_db()
        get_executable()

    with it("inserts and retrieves a row") as test:
        input_list = [
            "insert 1 user1 person1@example.com",
            "select",
            ".exit",
        ]

        responses = [
            "Execution Success!",
            "(1, user1, person1@example.com)",
            "Goodbye!"
        ]

        expected = [
            f"{EXEC_PREFIX}{input_list[0]}",
            f"{responses[0]}",
            f"{EXEC_PREFIX}{input_list[1]}",
            f"{responses[1]}",
            f"{responses[0]}",
            f"{EXEC_PREFIX}{input_list[2]}",
            f"{responses[2]}"
        ]

        result = run_executable(input_list)
        test.assertCountEqual(expected, result)

    # with it("prints error message when table is full") as test:
    #     input_list = []
    #
    #     for i in range(7802):
    #         input_list.append(f"insert {i} user{i} person{i}@example.com")
    #
    #     input_list.append(".exit")
    #
    #     result = run_executable(input_list)
    #
    #     test.assertIn("Table is full!", result)

    with it("prints error message when string input is too long") as test:
        long_username = ''.join(random.choice(string.ascii_uppercase + string.digits) for _ in range(33))
        long_email = ''.join(random.choice(string.ascii_uppercase + string.digits) for _ in range(256))

        input_list = [
            f"insert 1 {long_username} person1@example.com",
            f"insert 2 person {long_email}",
            ".exit",
        ]

        responses = [
            f'Could not prepare statement: "{input_list[0]}"',
            f'Could not prepare statement: "{input_list[1]}"',
            "Goodbye!"
        ]

        expected = [
            f"{EXEC_PREFIX}{input_list[0]}",
            f"{responses[0]}",
            f"{EXEC_PREFIX}{input_list[1]}",
            f"{responses[1]}",
            f"{EXEC_PREFIX}{input_list[2]}",
            f"{responses[2]}"
        ]

        result = run_executable(input_list)
        test.assertCountEqual(expected, result)

    with it("prints an error message if id is negative") as test:
        input_list = [
            "insert -1 user1 person1@example.com",
            "select",
            ".exit",
        ]

        responses = [
            f'Syntax error in: "{input_list[0]}"',
            "Execution Success!",
            "Goodbye!"
        ]

        expected = [
            f"{EXEC_PREFIX}{input_list[0]}",
            f"{responses[0]}",
            f"{EXEC_PREFIX}{input_list[1]}",
            f"{responses[1]}",
            f"{EXEC_PREFIX}{input_list[2]}",
            f"{responses[2]}"
        ]

        result = run_executable(input_list)
        test.assertCountEqual(expected, result)

    with it("keeps data after closing connection") as test:
        input_list1 = [
            "insert 1 user1 person1@example.com",
            ".exit",
        ]

        expected1 = [
            f"{EXEC_PREFIX}{input_list1[0]}",
            "Execution Success!",
            f"{EXEC_PREFIX}{input_list1[1]}",
            "Goodbye!"
        ]

        result1 = run_executable(input_list1)
        test.assertCountEqual(expected1, result1)

        input_list2 = [
            "select",
            ".exit",
        ]

        expected2 = [
            f"{EXEC_PREFIX}{input_list2[0]}",
            "(1, user1, person1@example.com)",
            "Execution Success!",
            f"{EXEC_PREFIX}{input_list2[1]}",
            "Goodbye!"
        ]

        result2 = run_executable(input_list2)
        print(result2)
        test.assertCountEqual(expected2, result2)

if __name__ == "__main__":
    get_executable()
    print(run_executable([".test", ".exit"]))
