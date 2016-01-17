# Trie tests guideline

Trie test input is an array of operations. Each operation must have 2 fields:

- `operation` - string, either `insert` or `remove`
- `key` - string, or hex value prefixed with `0x`

And optional field:

- `value`- which is used by `insert` operation

### Example

```json
{
	"input": 
	[
		{
			"operation": "insert",
			"key": "world",
			"value": "hello"
		},
		{
			"operation": "insert",
			"key": "0x1234",
			"value": "ooooops"
		},
		{
			"operation": "remove",
			"key": "0x1234"
		}
	],
	"output": "0x5991bb8c6514148a29db676a14ac506cd2cd5775ace63c30a4fe457715e9ac84"
}
```
