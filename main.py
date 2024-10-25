import datetime
import os
import random
import subprocess

# Constants
DATE_FORMAT = "%Y-%m-%d %H:%M:%S"
COMMIT_MESSAGES = [
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


def parse_date(date_str, default):
    return datetime.datetime.strptime(date_str, "%Y-%m-%d") if date_str else default


def set_random_time(hour, date):
    return date.replace(
        hour=hour,
        minute=random.randint(0, 59),
        second=random.randint(0, 59),
        microsecond=random.randint(0, 999999),
    )


def format_date(date):
    return date.strftime(DATE_FORMAT)


def write_to_file(filename, content):
    try:
        with open(filename, "w") as f:
            f.write(content + "\n")
    except IOError as e:
        print(f"Error writing to file {filename}: {e}")


def get_random_message():
    weighted_messages = [msg for msg, weight in COMMIT_MESSAGES for _ in range(weight)]
    return random.choice(weighted_messages)


def commit_changes(filename, date):
    formatted_date = format_date(date)
    try:
        subprocess.run(["git", "add", filename], check=True)
        os.environ["GIT_COMMITTER_DATE"] = formatted_date
        commit_msg = get_random_message()
        subprocess.run(["git", "commit", "--date", formatted_date, "-m", commit_msg], check=True)
    except subprocess.CalledProcessError as e:
        print(f"Error during git commit: {e}")
    finally:
        os.environ.pop("GIT_COMMITTER_DATE", None)


def is_weekend(date):
    return date.weekday() > 4


def flip_coin():
    return bool(random.randint(0, 1))


def perform_daily_commits(date, filename):
    if is_weekend(date) and not flip_coin():
        return  # Skip this weekend day

    daily_commits = (
        random.randint(0, 20) if not is_weekend(date) else random.randint(0, 8)
    )
    hours = sorted(random.sample(range(9, 21), daily_commits))
    for hour in hours:
        current_date = set_random_time(hour, date)
        write_to_file(filename, format_date(current_date))
        commit_changes(filename, current_date)


def write_commits(start="", end=""):
    try:
        subprocess.run(["git", "switch", "main"], check=True)
        subprocess.run(["git", "reset", "--hard", "dev"], check=True)
        subprocess.run(["git", "push", "--force"], check=True)
    except subprocess.CalledProcessError as e:
        print(f"Error during git operations: {e}")
        return

    one_year_ago = datetime.datetime.now() - datetime.timedelta(weeks=52)
    start_date = parse_date(start, one_year_ago)
    end_date = parse_date(end, datetime.datetime.now())

    while start_date < end_date:
        perform_daily_commits(start_date, "edit.txt")
        start_date += datetime.timedelta(days=1)
        if start_date.day % 10 == 0:  # Push every 10 days to avoid too many commits
            try:
                subprocess.run(["git", "push"], check=True)
            except subprocess.CalledProcessError as e:
                print(f"Error during git push: {e}")

    try:
        subprocess.run(["git", "push"], check=True)  # Push remaining commits
    except subprocess.CalledProcessError as e:
        print(f"Error during final git push: {e}")


def get_last_commit_info():
    try:
        last_commit_date = subprocess.check_output(["git", "log", "-1", "--format=%cd"]).decode().strip()
        last_commit_msg = subprocess.check_output(["git", "log", "-1", "--format=%s"]).decode().strip()
        last_commit_date = datetime.datetime.strptime(
            last_commit_date, "%a %b %d %H:%M:%S %Y %z"
        ).replace(tzinfo=None)
        return format_date(last_commit_date), last_commit_msg
    except subprocess.CalledProcessError as e:
        print(f"Error retrieving last commit info: {e}")
        return None, None


def catch_up():
    last_commit_date, _ = get_last_commit_info()
    if last_commit_date:
        end_date = format_date(datetime.datetime.now() + datetime.timedelta(days=1))
        write_commits(start=last_commit_date, end=end_date)


def main():
    _, last_commit_msg = get_last_commit_info()
    if last_commit_msg == "Add main.py":
        write_commits()
    else:
        catch_up()


if __name__ == "__main__":
    main()