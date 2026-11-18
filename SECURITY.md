# Security policy

win-care deletes files for a living, so we treat safety bugs as security bugs.

## Reporting a vulnerability

**Please do not open a public issue.** Report privately through
[GitHub private vulnerability reporting](https://github.com/esraa-emadeldin/win-care/security/advisories/new).

Include what you did, what happened, and the win-care and Windows versions. We aim to reply
within 7 days and to ship a fix for confirmed issues as fast as their severity calls for. We
credit reporters in the release notes unless you prefer otherwise.

## What counts

Anything that breaks the [safety model](docs/SAFETY.md), for example:

- a file deleted that the user did not see and tick in Review;
- a "move to Recycle Bin" that deletes permanently;
- a protected folder (Documents, Desktop, Windows, Program Files…) or something behind a
  junction/symlink reaching a plan or being changed;
- a way for another program or a web page to make win-care delete, run or elevate anything;
- a tampered update being accepted;
- any data leaving the PC.

## Supported versions

Only the latest release gets fixes while win-care is in 0.x.
