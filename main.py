import datetime
import os
import random
import subprocess
import logging

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

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

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
        logging.info(f"Wrote to file {filename}: {content}")
    except IOError as e:
        logging.error(f"Error writing to file {filename}: {e}")


def get_random_message():
    weighted_messages = [msg for msg, weight in COMMIT_MESSAGES for _ in range(weight)]
    return random.choice(weighted_messages)


def run_subprocess(command):
    try:
        result = subprocess.run(command, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        return result.stdout.decode().strip()
    except subprocess.CalledProcessError as e:
        logging.error(f"Error running command {' '.join(command)}: {e.stderr.decode().strip()}")
        raise


def commit_changes(filename, date):
    formatted_date = format_date(date)
    try:
        run_subprocess(["git", "add", filename])
        os.environ["GIT_COMMITTER_DATE"] = formatted_date
        commit_msg = get_random_message()
        run_subprocess(["git", "commit", "--date", formatted_date, "-m", commit_msg])
        logging.info(f"Committed changes with message: {commit_msg}")
    except subprocess.CalledProcessError as e:
        logging.error(f"Error during git commit: {e}")
    finally:
        os.environ.pop("GIT_COMMITTER_DATE", None)


def is_weekend(date):
    return date.weekday() > 4


def flip_coin():
    return bool(random.randint(0, 1))


def perform_daily_commits(date, filename):
    if is_weekend(date) and not flip_coin():
        logging.info(f"Skipping weekend day: {date}")
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
        run_subprocess(["git", "switch", "main"])
        run_subprocess(["git", "reset", "--hard", "dev"])
        run_subprocess(["git", "push", "--force"])
    except subprocess.CalledProcessError as e:
        logging.error(f"Error during git operations: {e}")
        return

    one_year_ago = datetime.datetime.now() - datetime.timedelta(weeks=52)
    start_date = parse_date(start, one_year_ago)
    end_date = parse_date(end, datetime.datetime.now())

    while start_date < end_date:
        perform_daily_commits(start_date, "edit.txt")
        start_date += datetime.timedelta(days=1)
        if start_date.day % 10 == 0:  # Push every 10 days to avoid too many commits
            try:
                run_subprocess(["git", "push"])
                logging.info("Pushed commits to remote repository")
            except subprocess.CalledProcessError as e:
                logging.error(f"Error during git push: {e}")

    try:
        run_subprocess(["git", "push"])  # Push remaining commits
        logging.info("Pushed remaining commits to remote repository")
    except subprocess.CalledProcessError as e:
        logging.error(f"Error during final git push: {e}")


def get_last_commit_info(branch):
    try:
        last_commit_date = run_subprocess(["git", "log", branch, "-1", "--format=%cd"])
        last_commit_msg = run_subprocess(["git", "log", branch, "-1", "--format=%s"])
        last_commit_date = datetime.datetime.strptime(
            last_commit_date, "%a %b %d %H:%M:%S %Y %z"
        ).replace(tzinfo=None)
        return format_date(last_commit_date), last_commit_msg
    except subprocess.CalledProcessError as e:
        logging.error(f"Error retrieving last commit info from {branch}: {e}")
        return None, None


def compare_last_commit_messages():
    _, main_commit_msg = get_last_commit_info("main")
    _, dev_commit_msg = get_last_commit_info("dev")
    return main_commit_msg == dev_commit_msg


def catch_up():
    last_commit_date, _ = get_last_commit_info("main")
    if last_commit_date:
        end_date = format_date(datetime.datetime.now() + datetime.timedelta(days=1))
        write_commits(start=last_commit_date, end=end_date)


def main():
    if compare_last_commit_messages():
        write_commits()
    else:
        catch_up()


if __name__ == "__main__":
    main()