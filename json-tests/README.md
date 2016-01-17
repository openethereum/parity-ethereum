# How to write json test file?

Cause it's very hard to write generic json test files, each subdirectory should follow its own
convention. BUT all json files `within` same directory should be consistent.

### Test files should always contain a single test with input and output.

```json
{
	input: ...,
	output: ...
}
```

As a reference, please use trietests.
