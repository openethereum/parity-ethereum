# Contributing Guidelines

## Do you have a question?

Check out our [Basic Usage](https://wiki.parity.io/Basic-Usage), [Configuration](https://wiki.parity.io/Configuring-Parity-Ethereum), and [FAQ](https://wiki.parity.io/FAQ) articles on our [wiki](https://wiki.parity.io/)!

See also frequently asked questions [tagged with `parity`](https://ethereum.stackexchange.com/questions/tagged/parity?sort=votes&pageSize=50) on Stack Exchange.

## Report bugs!

Do **not** open an issue on Github if you think your discovered bug could be a **security-relevant vulnerability**. Please, read our [security policy](../SECURITY.md) instead.

Otherwise, just create a [new issue](https://github.com/paritytech/parity-ethereum/issues/new) in our repository and state:

- What's your Parity Ethereum version?
- What's your operating system and version?
- How did you install Parity Ethereum?
- Is your node fully synchronized?
- Did you try turning it off and on again?

Also, try to include **steps to reproduce** the issue and expand on the **actual versus expected behavior**.

## Contribute!

If you would like to contribute to Parity Ethereum, please **fork it**, fix bugs or implement features, and [propose a pull request](https://github.com/paritytech/parity-ethereum/compare).

### Labels & Milestones

We use [labels](https://github.com/paritytech/parity-ethereum/labels) to manage PRs and issues and communicate the state of a PR. Please familiarize yourself with them. Furthermore we are organizing issues in [milestones](https://github.com/paritytech/parity-ethereum/milestones). Best way to get started is to a pick a ticket from the current milestone tagged [`easy`](https://github.com/paritytech/parity-ethereum/labels/Q2-easy%20%F0%9F%92%83) and get going, or [`mentor`](https://github.com/paritytech/parity-ethereum/labels/Q1-mentor%20%F0%9F%95%BA) and get in contact with the mentor offering their support on that larger task.

### Rules

There are a few basic ground-rules for contributors (including the maintainer(s) of the project):

* **No pushing directly to the master branch**.
* **All modifications** must be made in a **pull-request** to solicit feedback from other contributors.
* Pull-requests cannot be merged before CI runs green and two reviewers have given their approval.
* Contributors should adhere to the [Parity Ethereum Style Guide](https://wiki.parity.io/Parity-Ethereum-Style-Guide).

### Recommendations

* **Non-master branch names** *should* be prefixed with a short name moniker, followed by the associated Github Issue ID (if any), and a brief description of the task using the format `<GITHUB_USERNAME>-<ISSUE_ID>-<BRIEF_DESCRIPTION>` (e.g. `gavin-123-readme`). The name moniker helps people to inquiry about their unfinished work, and the GitHub Issue ID helps your future self and other developers (particularly those who are onboarding) find out about and understand the original scope of the task, and where it fits into Parity Ethereum [Projects](https://github.com/paritytech/parity-ethereum/projects).
* **Remove stale branches periodically**

### Preparing Pull Requests

* If your PR does not alter any logic (e.g. comments, dependencies, docs), then it may be tagged [`insubstantial`](https://github.com/paritytech/parity-ethereum/pulls?q=is%3Aopen+is%3Apr+label%3A%22A2-insubstantial+%F0%9F%91%B6%22).

* Once a PR is ready for review please add the [`pleasereview`](https://github.com/paritytech/parity-ethereum/pulls?utf8=%E2%9C%93&q=is%3Aopen+is%3Apr+label%3A%22A0-pleasereview+%F0%9F%A4%93%22+) label.

### Reviewing Pull Requests*:

* At least two reviewers are required to review PRs (even for PRs tagged [`insubstantial`](https://github.com/paritytech/parity-ethereum/pulls?q=is%3Aopen+is%3Apr+label%3A%22A2-insubstantial+%F0%9F%91%B6%22)).

When doing a review, make sure to look for any:

* Buggy behavior.
* Undue maintenance burden.
* Breaking with house coding style.
* Pessimization (i.e. reduction of speed as measured in the projects benchmarks).
* Breaking changes should be carefuly reviewed and tagged as such so they end up in the [changelog](../CHANGELOG.md).
* Uselessness (i.e. it does not strictly add a feature or fix a known issue).

## License.

By contributing to Parity Ethereum, you agree that your contributions will be licensed under the [GPLv3 License](../LICENSE).

Each contributor has to sign our Contributor License Agreement. The purpose of the CLA is to ensure that the guardian of a project's outputs has the necessary ownership or grants of rights over all contributions to allow them to distribute under the chosen license. You can read and sign our full Contributor License Agreement at [cla.parity.io](https://cla.parity.io) before submitting a pull request.
