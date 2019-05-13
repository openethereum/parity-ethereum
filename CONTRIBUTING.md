# Contributing

The `Parity Ethereum` project is an **OPENISH Open Source Project**

## What?

Individuals making significant and valuable contributions are given commit-access to a project to contribute as they see fit. A project is more like an open wiki than a standard guarded open source project.

## Rules

There are a few basic ground-rules for contributors (including the maintainer(s) of the project):

* **No `--force` pushes** or modifying the master branch history in any way. If you need to rebase, ensure you do it in your own repo.
* **Non-master branches**, prefixed with a short name moniker (e.g. `gav-my-feature`) must be used for ongoing work, and include the associated issue ID (if any) in the branch name. 
* **All modifications** must be made in a **pull-request** to solicit feedback from other contributors.
* A pull-request *must not be merged until CI* has finished successfully.
* Contributors should adhere to the [house coding style](https://wiki.parity.io/Coding-guide).


## Merge Process

Merging pull requests once CI is successful:

* A PR needs to be reviewed and approved by project maintainers unless:
	* it does not alter any logic (e.g. comments, dependencies, docs), then it may be tagged[`insubstantial`](https://github.com/paritytech/parity-ethereum/pulls?q=is%3Aopen+is%3Apr+label%3A%22A2-insubstantial+%F0%9F%91%B6%22) and merged by its author once CI is complete.
  * it is an urgent fix with no large change to logic, then it may be merged after a non-author contributor has approved the review once CI is complete.

* Once a PR is ready for review please add the [`pleasereview`](https://github.com/paritytech/parity-ethereum/pulls?utf8=%E2%9C%93&q=is%3Aopen+is%3Apr+label%3A%22A0-pleasereview+%F0%9F%A4%93%22+) label. Generally PRs should sit with this label for 48 hours in order to garner feedback. It may be merged before if all relevant parties had a look at it.
* No PR should be merged until all reviews' comments are addressed.

*Reviewing pull requests*:

When reviewing a pull request, the end-goal is to suggest useful changes to the author. Reviews should finish with approval unless there are issues that would result in:

* Buggy behavior.
* Undue maintenance burden.
* Breaking with house coding style.
* Pessimization (i.e. reduction of speed as measured in the projects benchmarks).
* Feature reduction (i.e. it removes some aspect of functionality that a significant minority of users rely on).
* Uselessness (i.e. it does not strictly add a feature or fix a known issue).

*Reviews may not be used as an effective veto for a PR because*:

* There exists a somewhat cleaner/better/faster way of accomplishing the same feature/fix.
* It does not fit well with some other contributors' longer-term vision for the project.

## Helping out

We use [labels](https://github.com/paritytech/parity-ethereum/labels) to manage PRs and issues and communicate state of a PR. Please familiarize yourself with them. Furthermore we are organizing issues in [milestones](https://github.com/paritytech/parity-ethereum/milestones). Best way to get started is to a pick a ticket from the current milestone tagged [`easy`](https://github.com/paritytech/parity-ethereum/labels/Q2-easy%20%F0%9F%92%83) and get going or [`mentor`](https://github.com/paritytech/parity-ethereum/labels/Q1-mentor%20%F0%9F%95%BA) and get in contact with the mentor offering their support on that larger task.

## Releases

Declaring formal releases remains the prerogative of the project maintainer(s).

## Changes to this arrangement

This is an experiment and feedback is welcome! This document may also be subject to pull-requests or changes by contributors where you believe you have something valuable to add or change.

## Heritage

These contributing guidelines are modified from the "OPEN Open Source Project" guidelines for the Level project: https://github.com/Level/community/blob/master/CONTRIBUTING.md
