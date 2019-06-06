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

* **No `--force` pushes** or modifying the master branch history in any way. If you need to rebase, ensure you do it in your own repo.
* **Non-master branches**, prefixed with a short name moniker (e.g. `gav-my-feature`) must be used for ongoing work, and include the associated issue ID (if any) in the branch name.
* **All modifications** must be made in a **pull-request** to solicit feedback from other contributors.
* A pull-request *must not be merged until CI* has finished successfully.
* Contributors should adhere to the [Parity Ethereum Style Guide](https://wiki.parity.io/Parity-Ethereum-Style-Guide).

### Merge Process

Merging pull requests once CI is successful:

* A PR needs to be reviewed and approved by project maintainers unless:
  * it does not alter any logic (e.g. comments, dependencies, docs), then it may be tagged [`insubstantial`](https://github.com/paritytech/parity-ethereum/pulls?q=is%3Aopen+is%3Apr+label%3A%22A2-insubstantial+%F0%9F%91%B6%22) and merged by its author once CI is complete.
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

## License.

By contributing to Parity Ethereum, you agree that your contributions will be licensed under the [GPLv3 License](../LICENSE).

Each contributor has to sign our Contributor License Agreement. The purpose of the CLA is to ensure that the guardian of a project's outputs has the necessary ownership or grants of rights over all contributions to allow them to distribute under the chosen license. You can read and sign our full Contributor License Agreement at [cla.parity.io](https://cla.parity.io) before submitting a pull request.
