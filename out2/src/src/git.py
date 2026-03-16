import os
import subprocess
from dataclasses import dataclass, field
from typing import List, Optional, Tuple

@dataclass
class CommitInfo:
    id: str
    author: str
    date: str
    summary: str

@dataclass
class CommitSummary:
    id: str
    summary: str
    author: str
    time: int

class GitRepo:
    class Error(Exception):
        pass

    def __init__(self, root: str):
        self.root = root

    @classmethod
    def discover(cls, root: str) -> Optional['GitRepo']:
        try:
            # Check if it's a git repo
            args = ['git', '-C', root, 'rev-parse', '--is-inside-work-tree']
            subprocess.check_output(args, stderr=subprocess.STDOUT)
            return cls(root)
        except (subprocess.CalledProcessError, FileNotFoundError):
            return None

    def uncommitted_file_count(self) -> int:
        args = ['git', '-C', self.root, 'status', '--porcelain']
        try:
            output = subprocess.check_output(args)
            lines = [l for l in output.decode('utf-8').split('\n') if l.strip()]
            return len(lines)
        except subprocess.CalledProcessError as e:
            raise self.Error(str(e)) from e

    def total_commit_count(self) -> int:
        args = ['git', '-C', self.root, 'rev-list', '--count', 'HEAD']
        try:
            output = subprocess.check_output(args)
            return int(output.decode('utf-8').strip())
        except subprocess.CalledProcessError as e:
            raise self.Error(str(e)) from e

    def recent_commits(self, limit: int) -> List[CommitSummary]:
        args = [
            'git', '-C', self.root, 'log',
            f'-{limit}',
            '--format=%h|%s|%an|%ct'
        ]
        try:
            output = subprocess.check_output(args)
            commits = []
            for line in output.decode('utf-8').strip().split('\n'):
                if not line:
                    continue
                parts = line.split('|', 3)
                if len(parts) == 4:
                    commits.append(CommitSummary(
                        id=parts[0],
                        summary=parts[1],
                        author=parts[2],
                        time=int(parts[3])
                    ))
            return commits
        except subprocess.CalledProcessError as e:
            raise self.Error(str(e)) from e

    def recent_commits_for_path(self, rel_path: str, limit: int) -> List[CommitSummary]:
        args = [
            'git', '-C', self.root, 'log',
            f'-{limit}',
            '--format=%h|%s|%an|%ct',
            '--follow',
            '--', rel_path
        ]
        try:
            output = subprocess.check_output(args)
            commits = []
            for line in output.decode('utf-8').strip().split('\n'):
                if not line:
                    continue
                parts = line.split('|', 3)
                if len(parts) == 4:
                    commits.append(CommitSummary(
                        id=parts[0],
                        summary=parts[1],
                        author=parts[2],
                        time=int(parts[3])
                    ))
            return commits
        except subprocess.CalledProcessError as e:
            raise self.Error(str(e)) from e

    def changed_files(self) -> List[str]:
        args = ['git', '-C', self.root, 'status', '--porcelain']
        try:
            output = subprocess.check_output(args)
            files = []
            for line in output.decode('utf-8').split('\n'):
                if len(line) > 2:
                    files.append(line[3:].strip())
            return sorted(set(files))
        except subprocess.CalledProcessError as e:
            raise self.Error(str(e)) from e

    def last_commit_info(self) -> CommitInfo:
        args = ['git', '-C', self.root, 'log', '-1', '--format=%H|%an|%ad|%s', '--date=iso-strict']
        try:
            output = subprocess.check_output(args)
            parts = output.decode('utf-8').strip().split('|')
            if len(parts) == 4:
                return CommitInfo(id=parts[0], author=parts[1], date=parts[2], summary=parts[3])
            return CommitInfo("", "", "", "")
        except subprocess.CalledProcessError:
            return CommitInfo("", "", "", "")

    def get_diff_stats(self, from_commit: str) -> Tuple[int, int]:
        args = ['git', '-C', self.root, 'diff', '--numstat', from_commit]
        try:
            output = subprocess.check_output(args)
            lines = output.decode('utf-8').split('\n')
            added = 0
            deleted = 0
            for line in lines:
                parts = line.split()
                if len(parts) >= 2:
                    if parts[0].isdigit():
                        added += int(parts[0])
                    if parts[1].isdigit():
                        deleted += int(parts[1])
            return added, deleted
        except subprocess.CalledProcessError as e:
            raise self.Error(str(e)) from e