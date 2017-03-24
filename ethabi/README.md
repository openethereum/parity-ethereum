# ethabi

[![Build Status][travis-image]][travis-url]

[travis-image]: https://travis-ci.org/paritytech/ethabi.svg?branch=master
[travis-url]: https://travis-ci.org/paritytech/ethabi

The ABI, Application Binary Interface, is basically how you call functions in a contract and get data back.

> An ABI determines such details as how functions are called and in which binary format information should be passed from one program component to the next...

An Ethereum smart contract is bytecode, EVM, on the Ethereum blockchain. Among the EVM, there could be several functions in a contract. An ABI is necessary so that you can specify which function in the contract to invoke, as well as get a guarantee that the function will return data in the format you are expecting. [read more](http://ethereum.stackexchange.com/a/1171/394)

This library encodes function calls and decodes their output.

[Documentation](http://paritytech.github.io/ethabi/ethabi/index.html)

### Installation

- via cargo

  ```
  cargo install ethabi --features="cli"
  ```

- via homebrew

  ```
  brew tap paritytech/paritytech
  brew install ethabi
  ```

### Usage

```
Ethereum ABI coder.
  Copyright 2016-2017 Parity Technologies (UK) Limited

Usage:
    ethabi encode function <abi-path> <function-name> [-p <param>]... [-l | --lenient]
    ethabi encode params [-v <type> <param>]... [-l | --lenient]
    ethabi decode function <abi-path> <function-name> <data>
    ethabi decode params [-t <type>]... <data>
    ethabi decode log <abi-path> <event-name> [-l <topic>]... <data>
    ethabi -h | --help

Options:
    -h, --help         Display this message and exit.
    -l, --lenient      Allow short representation of input params.

Commands:
    encode             Encode ABI call.
    decode             Decode ABI call result.
    function           Load function from json ABI file.
    params             Specify types of input params inline.
    log                Decode event log.
```

### Examples

```
ethabi encode params -v bool 1
```

> 0000000000000000000000000000000000000000000000000000000000000001

--

```
ethabi encode params -v bool 1 -v string gavofyork -v bool 0
```

> 00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000096761766f66796f726b0000000000000000000000000000000000000000000000

--

```
ethabi encode params -v bool[] [1,0,false]
```

> 00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000

--

```
ethabi encode function examples/test.json foo -p 1
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

--

```
ethabi decode params -t bool 0000000000000000000000000000000000000000000000000000000000000001
```

> bool true

--

```
ethabi decode params -t bool -t string -t bool 00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000096761766f66796f726b0000000000000000000000000000000000000000000000
```

> bool true<br/>
> string gavofyork<br/>
> bool false

--

```
ethabi decode params -t bool[] 00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```

> bool[] [true,false,false]

--

```
ethabi decode function ./examples/foo.json bar 0000000000000000000000000000000000000000000000000000000000000001
```

```json
[{
	"constant":false,
	"inputs":[{
		"name":"hello",
		"type":"address"
	}],
	"name":"bar",
	"outputs":[{
		"name":"",
		"type":"bool"
	}],
	"type":"function"
}]
```

> bool true

--

```
ethabi decode log ./examples/event.json Event -l 0000000000000000000000000000000000000000000000000000000000000001 0000000000000000000000004444444444444444444444444444444444444444
```

> a bool true<br/>
> b address 4444444444444444444444444444444444444444

### Alternative tools

-  [ethabi-js](https://github.com/jacogr/ethabi-js) - javascript port of this library created by [@jacogr](https://github.com/jacogr)

# Parity toolchain
*this project is a part of the parity toolchain*

- [**ethkey**](https://github.com/paritytech/ethkey) - Ethereum keys generator and signer.
- [**ethstore**](https://github.com/paritytech/ethstore) - Ethereum key management.
- [**ethabi**](https://github.com/paritytech/ethabi) - Ethereum function calls encoding.
