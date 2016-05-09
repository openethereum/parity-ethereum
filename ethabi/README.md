# ethabi

```
Ethereum ABI coder.
  Copyright 2016 Ethcore (UK) Limited

Usage:
    ethabi encode abi <abi-path> <function-name> [-p <param>]... [-l | --lenient]
    ethabi encode params [-p <type> <param>]... [-l | --lenient]
    ethabi decode abi <abi-path> <function-name> <data>
    ethabi decode params [-p <type>]... <data>
    ethabi -h | --help

Options:
    -h, --help         Display this message and exit.
    -l, --lenient      Allow short representation of input params.

Commands:
    encode             Encode ABI call.
    decode             Decode ABI call result.
    abi                Load json ABI from file.
    params             Specify types of input params inline.
```

### Examples

```
ethabi encode params -p bool 1
```

> 0000000000000000000000000000000000000000000000000000000000000001

```
ethabi encode params -p bool 1 -p string gavofyork -p bool 0
```

> 00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000096761766f66796f726b0000000000000000000000000000000000000000000000

```
ethabi encode abi examples/test.json foo -p 1
```

```json
[{
	"type":"function",
	"inputs": [{
		"name":"a",
		"type":"bool"
	}],
	"name":"foo",
	"outputs": []
}]
```

> 455575780000000000000000000000000000000000000000000000000000000000000001
