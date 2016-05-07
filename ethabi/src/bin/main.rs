extern crate ethabi;

fn print_help() {
	print!(r#"
Ethereum ABI coder.

Usage:
    ethabi <command> [<args>...]
    ethabi [options]

Options:
    -h, --help         Display this message.

Commands:
    encode             Encode ABI call.
    decode             Decode ABI call result.
"#);
}

fn print_encode_help() {
	print!(r#"
Encode ABI call.

Usage:
    ethabi encode abi <abi_path> <function_name> [<param>]
    ethabi encode params [-p <type> <param>]
    ethabi encode [options]

Options:
    -h, --help         Display this message.
"#);
}

fn print_decode_help() {
	print!(r#"
Decode ABI call result.

Usage:
    ethabi decode abi <abi_path> <function_name> <data>
    ethabi decode params [-p <type>] <data>
    ethabi decode [options]

Options:
    -h, --help         Display this message.
"#);
}




fn main() {
	print_decode_help();
}
