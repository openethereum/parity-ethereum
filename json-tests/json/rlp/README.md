# Rlp tests guideline

Rlp can be tested in various ways. It can encode/decode a value or an array of values. Let's start with encoding.

Each operation must have field:

- `operation` - `append`, `append_list`, `append_empty` or `append_raw`

Additionally `append` and `append_raw` must additionally define a `value` field:

- `value` - data

Also `append_raw` and `append_list` requires `len` field

- `len` - integer

### Encoding Test Example

```json
{
	"input":
	[
		{
			"operation": "append_list",
			"len": 2
		},
		{
			"operation": "append",
			"value": "cat"
		},
		{
			"operation": "append",
			"value": "dog"
		}
	]
	"output": "0xc88363617183646f67"
}
```

