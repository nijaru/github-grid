import datetime
import logging
import os
import random
import signal
import subprocess
import sys
import types
from contextlib import contextmanager
from typing import List, Optional, Set, Tuple

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
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
)


def parse_date(date_str: str, default: datetime.datetime) -> datetime.datetime:
    """Parse a date string into a datetime object."""
    return datetime.datetime.strptime(date_str, "%Y-%m-%d") if date_str else default


def set_random_time(hour: int, date: datetime.datetime) -> datetime.datetime:
    """Set a random time within a specific hour on a given date."""
    return date.replace(
        hour=hour,
        minute=random.randint(0, 59),
        second=random.randint(0, 59),
        microsecond=random.randint(0, 999999),
    )


def format_date(date: datetime.datetime) -> str:
    """Format a datetime object into a string."""
    return date.strftime(DATE_FORMAT)


def write_to_file(filename: str, content: str) -> None:
    """Write content to a file."""
    try:
        with open(filename, "w") as f:
            f.write(content + "\n")
        logging.info(f"Wrote to file {filename}: {content}")
    except IOError as e:
        logging.error(f"Error writing to file {filename}: {e}")


def get_random_message() -> str:
    """Get a random commit message based on weighted probabilities."""
    weighted_messages = [msg for msg, weight in COMMIT_MESSAGES for _ in range(weight)]
    return random.choice(weighted_messages)


def run_subprocess(command: List[str]) -> str:
    """Run a subprocess command and return its output."""
    try:
        result = subprocess.run(
            command, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        return result.stdout.decode().strip()
    except subprocess.CalledProcessError as e:
        logging.error(
            f"Error running command {' '.join(command)}: {e.stderr.decode().strip()}"
        )
        raise
    except Exception as e:
        logging.error(f"Unexpected error running command {' '.join(command)}: {e}")
        raise


@contextmanager
def set_git_commit_date(date: str):
    """Context manager to set the GIT_COMMITTER_DATE environment variable."""
    os.environ["GIT_COMMITTER_DATE"] = date
    try:
        yield
    finally:
        os.environ.pop("GIT_COMMITTER_DATE", None)


def commit_changes(filename: str, date: datetime.datetime) -> None:
    """Commit changes to the git repository."""
    formatted_date = format_date(date)
    try:
        run_subprocess(["git", "add", filename])
        commit_msg = get_random_message()
        with set_git_commit_date(formatted_date):
            run_subprocess(
                ["git", "commit", "--date", formatted_date, "-m", commit_msg]
            )
        logging.info(f"Committed changes with message: {commit_msg}")
    except subprocess.CalledProcessError as e:
        logging.error(f"Error during git commit: {e}")


def is_weekend(date: datetime.datetime) -> bool:
    """Check if a given date is a weekend."""
    return date.weekday() > 4


def flip_coin() -> bool:
    """Simulate a coin flip."""
    return bool(random.randint(0, 1))


def should_skip_day(date: datetime.datetime) -> bool:
    """Determine if the day should be skipped based on weekend and coin flip."""
    return is_weekend(date) and not flip_coin()


def generate_commit_times(
    date: datetime.datetime, daily_commits: int
) -> List[datetime.datetime]:
    """Generate a set of unique commit times for a given date."""
    commit_times: Set[datetime.datetime] = set()
    while len(commit_times) < daily_commits:
        hour = random.randint(9, 22)
        commit_time = set_random_time(hour, date)
        commit_times.add(commit_time)
    return sorted(commit_times)


def perform_daily_commits(date: datetime.datetime, filename: str) -> None:
    """Perform a number of commits for a given date."""
    if should_skip_day(date):
        logging.info(f"Skipping weekend day: {date.strftime('%Y-%m-%d')}")
        return  # Skip this weekend day

    daily_commits = (
        random.randint(0, 15) if not is_weekend(date) else random.randint(0, 3)
    )

    if daily_commits > 5 and flip_coin():
        daily_commits //= 2

    commit_times = generate_commit_times(date, daily_commits)

    for commit_time in commit_times:
        try:
            write_to_file(filename, format_date(commit_time))
            commit_changes(filename, commit_time)
        except Exception as e:
            logging.error(f"Error during commit process: {e}")

    logging.info(
        f"Performed {len(commit_times)} commits on {date.strftime('%Y-%m-%d')}"
    )


def push_commits() -> None:
    """Push commits to the remote repository."""
    try:
        run_subprocess(["git", "push"])
        logging.info("Pushed commits to remote repository")
    except subprocess.CalledProcessError as e:
        logging.error(f"Error during git push: {e}")


def write_commits(start: str = "", end: str = "") -> None:
    """Write commits between the start and end dates."""
    try:
        run_subprocess(["git", "switch", "main"])
        run_subprocess(["git", "reset", "--hard", "dev"])
        run_subprocess(["git", "push", "--force"])
    except subprocess.CalledProcessError as e:
        logging.error(f"Error during git operations: {e}")
        return

    one_year_ago_plus_one_week = datetime.datetime.now() - datetime.timedelta(weeks=53)
    start_date = parse_date(start, one_year_ago_plus_one_week)
    end_date = parse_date(end, datetime.datetime.now())

    while start_date < end_date:
        perform_daily_commits(start_date, "edit.txt")
        start_date += datetime.timedelta(days=1)
        if start_date.day % 10 == 0:  # Push every 10 days to avoid too many commits
            push_commits()

    push_commits()  # Push remaining commits


def get_last_commit_info(branch: str) -> Tuple[Optional[str], Optional[str]]:
    """Get the last commit date and message for a given branch."""
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


def compare_last_commit_messages() -> bool:
    """Compare the last commit messages of the main and dev branches."""
    _, main_commit_msg = get_last_commit_info("main")
    _, dev_commit_msg = get_last_commit_info("dev")
    return main_commit_msg == dev_commit_msg


def catch_up() -> None:
    """Catch up on commits if the main branch is behind."""
    last_commit_date, _ = get_last_commit_info("main")
    if last_commit_date:
        end_date = format_date(datetime.datetime.now() + datetime.timedelta(days=1))
        write_commits(start=last_commit_date, end=end_date)


def signal_handler(sig: int, frame: Optional[types.FrameType]) -> None:
    """Handle SIGINT (Ctrl+C) signal."""
    logging.info("Received SIGINT (Ctrl+C). Exiting gracefully...")
    sys.exit(0)


def main() -> None:
    """Main function to execute the script."""
    signal.signal(signal.SIGINT, signal_handler)  # Register the signal handler
    if compare_last_commit_messages():
        write_commits()
    else:
        catch_up()


if __name__ == "__main__":
    main()
