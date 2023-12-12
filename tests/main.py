import shutil

import pexpect
from ivoire import describe

EXEC_PREFIX = "db > "


class SqliteLite:
    pass


def get_executable():
    shutil.copy('../target/debug/sqlite_lite', '.')


def run_executable(inputs: list[str]):
    c = pexpect.spawnu('./sqlite_lite')
    final = ""

    for cmd in inputs:
        c.sendline(cmd)
        final += c.read_nonblocking(50000)

    resp = final.split("\n")

    resp = [r.strip() for r in resp if r]

    return resp

    # c.expect('db > ')
    # c.expect(pexpect.EOF)
    # c.expect(r'.+')
    # c.sendline(".test")
    # print(c.after)
    # print(c.before)
    # c.expect('db > ')
    # c.sendline(".test")

    # c.expect('\n')  # \r\n
    # print(c.before)
    # c.sendline(".exit")
    # print(c.before)

    # c.interact()
    # print(c.before)
    # print(c.readlines())


with describe(SqliteLite) as it:
    @it.before
    def before(_test):  # Takes at least 1 parameter, test, which can be used in other `it()`. Kinda like Object.self
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

    with it("prints error message when table is full") as test:
        input_list = []

        for i in range(7802):
            input_list.append(f"insert {i} user{i} person{i}@example.com")

        input_list.append(".exit")

        result = run_executable(input_list)

        print(result[-13:])
        test.assertIn("Table is full!", result, )

    # with it("prints error message when string input is too long") as test:
    #     assert False
    #     pass

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


if __name__ == "__main__":
    get_executable()
    print(run_executable([".test", ".exit"]))
