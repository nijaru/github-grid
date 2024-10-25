import datetime
import os
import random


def parse_start_date(start) -> datetime.datetime:
    if start:
        return datetime.datetime.strptime(start, "%Y-%m-%d")
    else:
        return datetime.datetime.now() - datetime.timedelta(weeks=53)


def parse_end_date(end) -> datetime.datetime:
    if end:
        return datetime.datetime.strptime(end, "%Y-%m-%d")
    else:
        # return the current date
        return datetime.datetime.now()


def set_time(hour, date) -> datetime.datetime:
    minute = random.randint(0, 59)
    second = random.randint(0, 59)
    microsecond = random.randint(0, 999999)
    date = date.replace(
        hour=hour, minute=minute, second=second, microsecond=microsecond
    )
    return date


def is_weekend(date) -> bool:
    return date.weekday() > 4


def flip_coin() -> bool:
    return bool(random.randint(0, 1))


def next_day(date) -> datetime.datetime:
    return date + datetime.timedelta(days=1)


def format_date(date) -> str:
    return date.strftime("%Y-%m-%d %H:%M:%S")


def write_date(filename, date) -> None:
    date = format_date(date)
    with open(filename, "w") as f:
        f.write(date + "\n")


def get_message(msg) -> str:
    messages = [
        ("Add a new feature", 10),
        ("Fix a bug", 8),
        ("Refactor some code", 6),
        ("Add a new test", 5),
        ("Update the requirements", 4),
        ("Update the documentation", 3),
        ("Update the README", 3),
        ("Update the license", 2),
        ("Update the gitignore", 1),
        ("Update the CI/CD pipeline", 1),
        ("Update the Dockerfile", 1),
        ("Update the Makefile", 1),
        ("Update the GitHub Actions", 1),
        ("Update the Jenkinsfile", 1),
        ("Update the AWS config", 1),
        ("Update the GCP config", 1),
        ("Update the Azure config", 1),
    ]
    weighted_messages = [message for message, weight in messages for _ in range(weight)]
    new_msg = random.choice(weighted_messages)
    return new_msg


def commit(filename, date, msg) -> None:
    date = format_date(date)
    os.system(f"git add {filename}")
    os.environ["GIT_COMMITTER_DATE"] = date
    msg = get_message(msg)
    rebase_cmd = f"git commit --date='{date}' -m '{msg}'"
    os.system(rebase_cmd)


def work(i, date, filename, msg):
    current_date = set_time(i, date)
    write_date(filename, current_date)
    commit(filename, current_date, msg)


def write_commits(start="", end=""):
    os.system("git switch main")
    os.system("git reset --hard dev")
    os.system("git push --force")

    date = parse_start_date(start)
    end = parse_end_date(end)

    while date < end:
        filename = "edit.txt"
        msg = ""
        commits = 0

        daily_commits = random.randint(0,20)

        if is_weekend(date) and flip_coin():
            daily_commits = random.randint(0, 8)
        else:
            continue

        hours = random.sample(range(9, 21), daily_commits)
        for i in sorted(hours):
            work(i, date, filename, msg)
            commits += 1

        date = next_day(date)
        if commits > 100:
            os.system("git push")
            commits = 0

    # push the last commits
    os.system("git push")


def get_last_commit_date():
    last_commit_date = os.popen("git log -1 --format=%cd").read().strip()
    last_commit_date = datetime.datetime.strptime(
        last_commit_date, "%a %b %d %H:%M:%S %Y %z"
    ).replace(tzinfo=None)
    last_commit_date = format_date(last_commit_date)
    return last_commit_date


def get_last_commit_msg():
    return os.popen("git log -1 --format=%s").read().strip()


def catch_up():
    last_commit_date = get_last_commit_date()
    end = datetime.datetime.now() + datetime.timedelta(days=1)
    end = format_date(end)
    # todo fix date parsing
    write_commits(start=last_commit_date, end=end)


def main():
    last_commit_msg = get_last_commit_msg()
    if last_commit_msg == "Add main.py":
        write_commits()
    else:
        catch_up()



if __name__ == "__main__":
    main()
