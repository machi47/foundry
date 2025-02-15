use super::{ClapChain, EthereumOpts, Wallet};
use crate::{
    cmd::cast::{find_block::FindBlockArgs, run::RunArgs},
    utils::{parse_ether_value, parse_u256},
};
use clap::{Parser, Subcommand, ValueHint};
use ethers::types::{Address, BlockId, BlockNumber, NameOrAddress, H256, U256};
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Subcommand)]
#[clap(
    about = "Perform Ethereum RPC calls from the comfort of your command line.",
    after_help = "Find more information in the book: http://book.getfoundry.sh/reference/cast/cast.html"
)]
pub enum Subcommands {
    #[clap(name = "--max-int")]
    #[clap(aliases = &["max-int", "maxi"])]
    #[clap(about = "Get the maximum i256 value.")]
    MaxInt,
    #[clap(name = "--min-int")]
    #[clap(aliases = &["min-int", "mini"])]
    #[clap(about = "Get the minimum i256 value.")]
    MinInt,
    #[clap(name = "--max-uint")]
    #[clap(aliases = &["max-uint", "maxu"])]
    #[clap(about = "Get the maximum u256 value.")]
    MaxUint,
    #[clap(name = "--address-zero", about = "Get zero address")]
    #[clap(aliases = &["address-zero", "az"])]
    AddressZero,
    #[clap(name = "--hash-zero", about = "Get zero hash")]
    #[clap(aliases = &["hash-zero", "hz"])]
    HashZero,
    #[clap(name = "--from-utf8")]
    #[clap(aliases = &["from-utf8", "--from-ascii", "from-ascii", "fu", "fa"])]
    #[clap(about = "Convert UTF8 text to hex.")]
    FromUtf8 {
        #[clap(value_name = "TEXT")]
        text: Option<String>,
    },
    #[clap(name = "--to-hex")]
    #[clap(aliases = &["to-hex", "th", "2h"])]
    #[clap(about = "Convert an integer to hex.")]
    ToHex {
        #[clap(value_name = "DECIMAL")]
        decimal: Option<String>,
    },
    #[clap(name = "--concat-hex")]
    #[clap(aliases = &["concat-hex", "ch"])]
    #[clap(about = "Concatenate hex strings.")]
    ConcatHex {
        #[clap(value_name = "DATA")]
        data: Vec<String>,
    },
    #[clap(name = "--from-bin")]
    #[clap(aliases = &["from-bin", "fb"])]
    #[clap(about = "Convert binary data into hex data.")]
    FromBin,
    #[clap(name = "--to-hexdata")]
    #[clap(aliases = &["to-hexdata", "thd", "2hd"])]
    #[clap(
        about = "Normalize the input to lowercase, 0x-prefixed hex. See --help for more info.",
        long_about = r#"Normalize the input to lowercase, 0x-prefixed hex.

The input can be:
- mixed case hex with or without 0x prefix
- 0x prefixed hex, concatenated with a ':'
- an absolute path to file
- @tag, where the tag is defined in an environment variable"#
    )]
    ToHexdata {
        #[clap(value_name = "INPUT")]
        input: Option<String>,
    },
    #[clap(name = "--to-checksum-address")]
    #[clap(aliases = &["to-checksum-address", "--to-checksum", "to-checksum", "ta", "2a"])] // Compatibility with dapptools' cast
    #[clap(about = "Convert an address to a checksummed format (EIP-55).")]
    ToCheckSumAddress {
        #[clap(value_name = "ADDRESS")]
        address: Option<Address>,
    },
    #[clap(name = "--to-ascii")]
    #[clap(aliases = &["to-ascii", "tas", "2as"])]
    #[clap(about = "Convert hex data to an ASCII string.")]
    ToAscii {
        #[clap(value_name = "HEXDATA")]
        hexdata: Option<String>,
    },
    #[clap(name = "--from-fix")]
    #[clap(aliases = &["from-fix", "ff"])]
    #[clap(about = "Convert a fixed point number into an integer.")]
    FromFix {
        #[clap(value_name = "DECIMALS")]
        decimals: Option<u128>,
        #[clap(allow_hyphen_values = true, value_name = "VALUE")]
        // negative values not yet supported internally
        value: Option<String>,
    },
    #[clap(name = "--to-bytes32")]
    #[clap(aliases = &["to-bytes32", "tb", "2b"])]
    #[clap(about = "Right-pads hex data to 32 bytes.")]
    ToBytes32 {
        #[clap(value_name = "BYTES")]
        bytes: Option<String>,
    },
    #[clap(name = "--to-dec")]
    #[clap(aliases = &["to-dec", "td", "2d"])]
    #[clap(about = "Convert hex value into a decimal number.")]
    ToDec {
        #[clap(value_name = "HEXVALUE")]
        hexvalue: Option<String>,
    },
    #[clap(name = "--to-fix")]
    #[clap(aliases = &["to-fix", "tf", "2f"])]
    #[clap(about = "Convert an integer into a fixed point number.")]
    ToFix {
        #[clap(value_name = "DECIMALS")]
        decimals: Option<u128>,
        #[clap(allow_hyphen_values = true, value_name = "VALUE")]
        // negative values not yet supported internally
        value: Option<String>,
    },
    #[clap(name = "--to-uint256")]
    #[clap(aliases = &["to-uint256", "tu", "2u"])]
    #[clap(about = "Convert a number to a hex-encoded uint256.")]
    ToUint256 {
        #[clap(value_name = "VALUE")]
        value: Option<String>,
    },
    #[clap(name = "--to-int256")]
    #[clap(aliases = &["to-int256", "ti", "2i"])]
    #[clap(about = "Convert a number to a hex-encoded int256.")]
    ToInt256 {
        #[clap(value_name = "VALUE")]
        value: Option<String>,
    },
    #[clap(name = "--to-unit")]
    #[clap(aliases = &["to-unit", "tun", "2un"])]
    #[clap(
        about = "Convert an ETH amount into another unit (ether, gwei or wei).",
        long_about = r#"Convert an ETH amount into another unit (ether, gwei or wei).\

Examples:
- 1ether wei
- "1 ether" wei
- 1ether
- 1 gwei
- 1gwei ether"#
    )]
    ToUnit {
        #[clap(value_name = "VALUE")]
        value: Option<String>,
        #[clap(
            help = "The unit to convert to (ether, gwei, wei).",
            default_value = "wei",
            value_name = "UNIT"
        )]
        unit: String,
    },
    #[clap(name = "--to-wei")]
    #[clap(aliases = &["to-wei", "tw", "2w"])]
    #[clap(about = "Convert an ETH amount to wei. Consider using --to-unit.")]
    ToWei {
        #[clap(allow_hyphen_values = true, value_name = "VALUE")]
        // negative values not yet supported internally
        value: Option<String>,
        #[clap(value_name = "UNIT")]
        unit: Option<String>,
    },
    #[clap(name = "--from-wei")]
    #[clap(aliases = &["from-wei", "fw"])]
    #[clap(about = "Convert wei into an ETH amount. Consider using --to-unit.")]
    FromWei {
        #[clap(allow_hyphen_values = true, value_name = "VALUE")]
        // negative values not yet supported internally
        value: Option<String>,
        #[clap(value_name = "UNIT")]
        unit: Option<String>,
    },
    #[clap(name = "access-list")]
    #[clap(aliases = &["ac", "acl"])]
    #[clap(about = "Create an access list for a transaction.")]
    AccessList {
        #[clap(help = "The destination of the transaction.", parse(try_from_str = parse_name_or_address), value_name = "ADDRESS")]
        address: NameOrAddress,
        #[clap(help = "The signature of the function to call.", value_name = "SIG")]
        sig: String,
        #[clap(help = "The arguments of the function to call.", value_name = "ARGS")]
        args: Vec<String>,
        #[clap(
            long,
            short = 'B',
            help = "The block height you want to query at.",
            long_help = "The block height you want to query at. Can also be the tags earliest, latest, or pending.",
            parse(try_from_str = parse_block_id),
            value_name = "BLOCK"
        )]
        block: Option<BlockId>,
        #[clap(flatten)]
        // TODO: We only need RPC URL + etherscan stuff from this struct
        eth: EthereumOpts,
        #[clap(long = "json", short = 'j', help_heading = "DISPLAY OPTIONS")]
        to_json: bool,
    },
    #[clap(name = "block")]
    #[clap(alias = "bl")]
    #[clap(about = "Get information about a block.")]
    Block {
        #[clap(
            long,
            short = 'B',
            help = "The block height you want to query at.",
            long_help = "The block height you want to query at. Can also be the tags earliest, latest, or pending.",
            parse(try_from_str = parse_block_id),
            value_name = "BLOCK"
        )]
        block: BlockId,
        #[clap(long, env = "CAST_FULL_BLOCK")]
        full: bool,
        #[clap(
            long,
            short,
            help = "If specified, only get the given field of the block.",
            value_name = "FIELD"
        )]
        field: Option<String>,
        #[clap(long = "json", short = 'j', help_heading = "DISPLAY OPTIONS")]
        to_json: bool,
        #[clap(long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "block-number")]
    #[clap(alias = "bn")]
    #[clap(about = "Get the latest block number.")]
    BlockNumber {
        #[clap(long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "call")]
    #[clap(alias = "c")]
    #[clap(about = "Perform a call on an account without publishing a transaction.")]
    Call {
        #[clap(help = "the address you want to query", parse(try_from_str = parse_name_or_address), value_name = "ADDRESS")]
        address: NameOrAddress,
        #[clap(value_name = "SIG")]
        sig: String,
        #[clap(value_name = "ARGS")]
        args: Vec<String>,
        #[clap(long, short, help = "the block you want to query, can also be earliest/latest/pending", parse(try_from_str = parse_block_id), value_name = "BLOCK")]
        block: Option<BlockId>,
        #[clap(flatten)]
        eth: EthereumOpts,
    },
    #[clap(alias = "cd")]
    #[clap(about = "ABI-encode a function with arguments.")]
    Calldata {
        #[clap(
            help = "The function signature.",
            long_help = "The function signature in the form <name>(<types...>)",
            value_name = "SIG"
        )]
        sig: String,
        #[clap(allow_hyphen_values = true, value_name = "ARGS")]
        args: Vec<String>,
    },
    #[clap(name = "chain")]
    #[clap(alias = "ch")]
    #[clap(about = "Get the symbolic name of the current chain.")]
    Chain {
        #[clap(long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "chain-id")]
    #[clap(aliases = &["ci", "cid"])]
    #[clap(about = "Get the Ethereum chain ID.")]
    ChainId {
        #[clap(long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "client")]
    #[clap(alias = "cl")]
    #[clap(about = "Get the current client version.")]
    Client {
        #[clap(long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "compute-address")]
    #[clap(alias = "ca")]
    #[clap(about = "Compute the contract address from a given nonce and deployer address.")]
    ComputeAddress {
        #[clap(long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
        #[clap(help = "The deployer address.", value_name = "ADDRESS")]
        address: String,
        #[clap(long, help = "The nonce of the deployer address.", parse(try_from_str = parse_u256), value_name = "NONCE")]
        nonce: Option<U256>,
    },
    #[clap(name = "namehash")]
    #[clap(aliases = &["na", "nh"])]
    #[clap(about = "Calculate the ENS namehash of a name.")]
    Namehash {
        #[clap(value_name = "NAME")]
        name: String,
    },
    #[clap(name = "tx")]
    #[clap(alias = "t")]
    #[clap(about = "Get information about a transaction.")]
    Tx {
        #[clap(value_name = "HASH")]
        hash: String,
        #[clap(value_name = "FIELD")]
        field: Option<String>,
        #[clap(long = "json", short = 'j', help_heading = "DISPLAY OPTIONS")]
        to_json: bool,
        #[clap(long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "receipt")]
    #[clap(alias = "re")]
    #[clap(about = "Get the transaction receipt for a transaction.")]
    Receipt {
        #[clap(value_name = "TX_HASH")]
        hash: String,
        #[clap(value_name = "FIELD")]
        field: Option<String>,
        #[clap(
            short,
            long,
            help = "The number of confirmations until the receipt is fetched",
            default_value = "1",
            value_name = "CONFIRMATIONS"
        )]
        confirmations: usize,
        #[clap(long, env = "CAST_ASYNC")]
        cast_async: bool,
        #[clap(long = "json", short = 'j', help_heading = "DISPLAY OPTIONS")]
        to_json: bool,
        #[clap(long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "send")]
    #[clap(alias = "s")]
    #[clap(about = "Sign and publish a transaction.")]
    SendTx {
        #[clap(
            help = "The destination of the transaction.",
            parse(try_from_str = parse_name_or_address),
            value_name = "TO"
        )]
        to: NameOrAddress,
        #[clap(help = "The signature of the function to call.", value_name = "SIG")]
        sig: Option<String>,
        #[clap(help = "The arguments of the function to call.", value_name = "ARGS")]
        args: Vec<String>,
        #[clap(long, help = "Gas limit for the transaction.", parse(try_from_str = parse_u256), value_name = "GAS")]
        gas: Option<U256>,
        #[clap(
            long = "gas-price",
            help = "Gas price for legacy transactions, or max fee per gas for EIP1559 transactions.",
            env = "ETH_GAS_PRICE",
            parse(try_from_str = parse_ether_value),
            value_name = "PRICE"
        )]
        gas_price: Option<U256>,
        #[clap(
            long,
            help = "Ether to send in the transaction.",
            long_help = r#"Ether to send in the transaction, either specified in wei, or as a string with a unit type.

Examples: 1ether, 10gwei, 0.01ether"#,
            parse(try_from_str = parse_ether_value),
            value_name = "VALUE"
        )]
        value: Option<U256>,
        #[clap(long, help = "nonce for the transaction", parse(try_from_str = parse_u256), value_name = "NONCE")]
        nonce: Option<U256>,
        #[clap(long, env = "CAST_ASYNC")]
        cast_async: bool,
        #[clap(flatten)]
        eth: EthereumOpts,
        #[clap(
            long,
            help = "Send a legacy transaction instead of an EIP1559 transaction.",
            long_help = r#"Send a legacy transaction instead of an EIP1559 transaction.

This is automatically enabled for common networks without EIP1559."#
        )]
        legacy: bool,
        #[clap(
            short,
            long,
            help = "The number of confirmations until the receipt is fetched.",
            default_value = "1",
            value_name = "CONFIRMATIONS"
        )]
        confirmations: usize,
        #[clap(long = "json", short = 'j', help_heading = "DISPLAY OPTIONS")]
        to_json: bool,
        #[clap(
            long = "resend",
            help = "Reuse the latest nonce for the sender account.",
            conflicts_with = "nonce"
        )]
        resend: bool,
    },
    #[clap(name = "publish")]
    #[clap(alias = "p")]
    #[clap(about = "Publish a raw transaction to the network.")]
    PublishTx {
        #[clap(help = "The raw transaction", value_name = "RAW_TX")]
        raw_tx: String,
        #[clap(long, env = "CAST_ASYNC")]
        cast_async: bool,
        // FIXME: We only need the RPC URL and `--flashbots` options from this.
        #[clap(flatten)]
        eth: EthereumOpts,
    },
    #[clap(name = "estimate")]
    #[clap(alias = "e")]
    #[clap(about = "Estimate the gas cost of a transaction.")]
    Estimate {
        #[clap(help = "The destination of the transaction.", parse(try_from_str = parse_name_or_address), value_name = "TO")]
        to: NameOrAddress,
        #[clap(help = "The signature of the function to call.", value_name = "SIG")]
        sig: String,
        #[clap(help = "The arguments of the function to call.", value_name = "ARGS")]
        args: Vec<String>,
        #[clap(
            long,
            help = "Ether to send in the transaction.",
            long_help = r#"Ether to send in the transaction, either specified in wei, or as a string with a unit type.

Examples: 1ether, 10gwei, 0.01ether"#,
            parse(try_from_str = parse_ether_value),
            value_name = "VALUE"
        )]
        value: Option<U256>,
        #[clap(flatten)]
        // TODO: We only need RPC URL and Etherscan API key here.
        eth: EthereumOpts,
    },
    #[clap(name = "--calldata-decode")]
    #[clap(alias = "cdd")]
    #[clap(about = "Decode ABI-encoded input data.")]
    CalldataDecode {
        #[clap(
            help = "The function signature in the format `<name>(<in-types>)(<out-types>)`.",
            value_name = "SIG"
        )]
        sig: String,
        #[clap(help = "The ABI-encoded calldata.", value_name = "CALLDATA")]
        calldata: String,
    },
    #[clap(name = "--abi-decode")]
    #[clap(alias = "ad")]
    #[clap(
        about = "Decode ABI-encoded input or output data",
        long_about = r#"Decode ABI-encoded input or output data.

Defaults to decoding output data. To decode input data pass --input or use cast --calldata-decode."#
    )]
    AbiDecode {
        #[clap(
            help = "The function signature in the format `<name>(<in-types>)(<out-types>)`.",
            value_name = "SIG"
        )]
        sig: String,
        #[clap(help = "The ABI-encoded calldata.", value_name = "CALLDATA")]
        calldata: String,
        #[clap(long, short, help = "Decode input data.")]
        input: bool,
    },
    #[clap(name = "abi-encode")]
    #[clap(alias = "ae")]
    #[clap(about = "ABI encode the given function argument, excluding the selector.")]
    AbiEncode {
        #[clap(help = "The function signature.", value_name = "SIG")]
        sig: String,
        #[clap(help = "The arguments of the function.", value_name = "ARGS")]
        #[clap(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[clap(name = "index")]
    #[clap(alias = "in")]
    #[clap(about = "Compute the storage slot for an entry in a mapping.")]
    Index {
        #[clap(help = "The mapping key type.", value_name = "KEY_TYPE")]
        key_type: String,
        #[clap(help = "The mapping value type.", value_name = "VALUE_TYPE")]
        value_type: String,
        #[clap(help = "The mapping key.", value_name = "KEY")]
        key: String,
        #[clap(help = "The storage slot of the mapping.", value_name = "SLOT_NUMBER")]
        slot_number: String,
    },
    #[clap(name = "4byte")]
    #[clap(aliases = &["4", "4b"])]
    #[clap(
        about = "Get the function signatures for the given selector from https://sig.eth.samczsun.com."
    )]
    FourByte {
        #[clap(help = "The function selector.", value_name = "SELECTOR")]
        selector: String,
    },
    #[clap(name = "4byte-decode")]
    #[clap(aliases = &["4d", "4bd"])]
    #[clap(about = "Decode ABI-encoded calldata using https://sig.eth.samczsun.com.")]
    FourByteDecode {
        #[clap(help = "The ABI-encoded calldata.", value_name = "CALLDATA")]
        calldata: String,
    },
    #[clap(name = "4byte-event")]
    #[clap(aliases = &["4e", "4be"])]
    #[clap(
        about = "Get the event signature for a given topic 0 from https://sig.eth.samczsun.com."
    )]
    FourByteEvent {
        #[clap(help = "Topic 0", value_name = "TOPIC_0")]
        topic: String,
    },
    #[clap(name = "upload-signature")]
    #[clap(aliases = &["ups"])]
    #[clap(about = r#"Upload the given signatures to https://sig.eth.samczsun.com.

    Examples:
    - cast upload-signature "transfer(address,uint256)"
    - cast upload-signature "function transfer(address,uint256)"
    - cast upload-signature "function transfer(address,uint256)" "event Transfer(address,address,uint256)"
    - cast upload-signature ./out/Contract.sol/Contract.json
    "#)]
    UploadSignature {
        #[clap(
            help = "The signatures to upload. Prefix with 'function', 'event', or 'error'. Defaults to function if no prefix given. Can also take paths to contract artifact JSON."
        )]
        signatures: Vec<String>,
    },
    #[clap(name = "pretty-calldata")]
    #[clap(alias = "pc")]
    #[clap(
        about = "Pretty print calldata.",
        long_about = r#"Pretty print calldata.

Tries to decode the calldata using https://sig.eth.samczsun.com unless --offline is passed."#
    )]
    PrettyCalldata {
        #[clap(help = "The calldata.", value_name = "CALLDATA")]
        calldata: String,
        #[clap(long, short, help = "Skip the https://sig.eth.samczsun.com lookup.")]
        offline: bool,
    },
    #[clap(name = "age")]
    #[clap(alias = "a")]
    #[clap(about = "Get the timestamp of a block.")]
    Age {
        #[clap(
            long,
            short = 'B',
            help = "The block height you want to query at.",
            long_help = "The block height you want to query at. Can also be the tags earliest, latest, or pending.",
            parse(try_from_str = parse_block_id),
            value_name = "BLOCK"
        )]
        block: Option<BlockId>,
        #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "balance")]
    #[clap(alias = "b")]
    #[clap(about = "Get the balance of an account in wei.")]
    Balance {
        #[clap(
            long,
            short = 'B',
            help = "The block height you want to query at.",
            long_help = "The block height you want to query at. Can also be the tags earliest, latest, or pending.",
            parse(try_from_str = parse_block_id),
            value_name = "BLOCK"
        )]
        block: Option<BlockId>,
        #[clap(help = "The account you want to query", parse(try_from_str = parse_name_or_address), value_name = "WHO")]
        who: NameOrAddress,
        #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "basefee")]
    #[clap(aliases = &["ba", "fee"])]
    #[clap(about = "Get the basefee of a block.")]
    BaseFee {
        #[clap(
            long,
            short = 'B',
            help = "The block height you want to query at.",
            long_help = "The block height you want to query at. Can also be the tags earliest, latest, or pending.",
            parse(try_from_str = parse_block_id),
            value_name = "BLOCK"
        )]
        block: Option<BlockId>,
        #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "code")]
    #[clap(alias = "co")]
    #[clap(about = "Get the bytecode of a contract.")]
    Code {
        #[clap(
            long,
            short = 'B',
            help = "The block height you want to query at.",
            long_help = "The block height you want to query at. Can also be the tags earliest, latest, or pending.",
            parse(try_from_str = parse_block_id),
            value_name = "BLOCK"
        )]
        block: Option<BlockId>,
        #[clap(help = "The contract address.", parse(try_from_str = parse_name_or_address), value_name = "WHO")]
        who: NameOrAddress,
        #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "gas-price")]
    #[clap(alias = "g")]
    #[clap(about = "Get the current gas price.")]
    GasPrice {
        #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "keccak")]
    #[clap(alias = "k")]
    #[clap(about = "Hash arbitrary data using keccak-256.")]
    Keccak {
        #[clap(value_name = "DATA")]
        data: String,
    },
    #[clap(name = "resolve-name")]
    #[clap(alias = "rn")]
    #[clap(about = "Perform an ENS lookup.")]
    ResolveName {
        #[clap(help = "The name to lookup.", value_name = "WHO")]
        who: Option<String>,
        #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
        #[clap(long, short, help = "Perform a reverse lookup to verify that the name is correct.")]
        verify: bool,
    },
    #[clap(name = "lookup-address")]
    #[clap(alias = "l")]
    #[clap(about = "Perform an ENS reverse lookup.")]
    LookupAddress {
        #[clap(help = "The account to perform the lookup for.", value_name = "WHO")]
        who: Option<Address>,
        #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
        #[clap(
            long,
            short,
            help = "Perform a normal lookup to verify that the address is correct."
        )]
        verify: bool,
    },
    #[clap(
        name = "storage",
        alias = "st",
        about = "Get the raw value of a contract's storage slot."
    )]
    Storage {
        #[clap(help = "The contract address.", parse(try_from_str = parse_name_or_address), value_name = "ADDRESS")]
        address: NameOrAddress,
        #[clap(help = "The storage slot number (hex or decimal)", parse(try_from_str = parse_slot), value_name = "SLOT")]
        slot: H256,
        #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
        #[clap(
            long,
            short = 'B',
            help = "The block height you want to query at.",
            long_help = "The block height you want to query at. Can also be the tags earliest, latest, or pending.",
            parse(try_from_str = parse_block_id),
            value_name = "BLOCK"
        )]
        block: Option<BlockId>,
    },
    #[clap(
        name = "proof",
        alias = "pr",
        about = "Generate a storage proof for a given storage slot."
    )]
    Proof {
        #[clap(help = "The contract address.", parse(try_from_str = parse_name_or_address), value_name = "ADDRESS")]
        address: NameOrAddress,
        #[clap(help = "The storage slot numbers (hex or decimal).", parse(try_from_str = parse_slot), value_name = "SLOTS")]
        slots: Vec<H256>,
        #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
        #[clap(
            long,
            short = 'B',
            help = "The block height you want to query at.",
            long_help = "The block height you want to query at. Can also be the tags earliest, latest, or pending.",
            parse(try_from_str = parse_block_id),
            value_name = "BLOCK"
        )]
        block: Option<BlockId>,
    },
    #[clap(name = "nonce")]
    #[clap(alias = "n")]
    #[clap(about = "Get the nonce for an account.")]
    Nonce {
        #[clap(
            long,
            short = 'B',
            help = "The block height you want to query at.",
            long_help = "The block height you want to query at. Can also be the tags earliest, latest, or pending.",
            parse(try_from_str = parse_block_id),
            value_name = "BLOCK"
        )]
        block: Option<BlockId>,
        #[clap(help = "The address you want to get the nonce for.", parse(try_from_str = parse_name_or_address), value_name = "WHO")]
        who: NameOrAddress,
        #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
        rpc_url: Option<String>,
    },
    #[clap(name = "etherscan-source")]
    #[clap(aliases = &["et", "src"])]
    #[clap(about = "Get the source code of a contract from Etherscan.")]
    EtherscanSource {
        #[clap(flatten)]
        chain: ClapChain,
        #[clap(help = "The contract's address.", value_name = "ADDRESS")]
        address: String,
        #[clap(short, help = "The output directory to expand source tree into.", value_hint = ValueHint::DirPath, value_name = "DIRECTORY")]
        directory: Option<PathBuf>,
        #[clap(long, env = "ETHERSCAN_API_KEY", value_name = "KEY")]
        etherscan_api_key: Option<String>,
    },
    #[clap(name = "wallet", alias = "w", about = "Wallet management utilities.")]
    Wallet {
        #[clap(subcommand)]
        command: WalletSubcommands,
    },
    #[clap(
        name = "interface",
        alias = "i",
        about = "Generate a Solidity interface from a given ABI.",
        long_about = "Generate a Solidity interface from a given ABI. Currently does not support ABI encoder v2."
    )]
    Interface {
        #[clap(
            help = "The contract address, or the path to an ABI file.",
            long_help = r#"The contract address, or the path to an ABI file.

If an address is specified, then the ABI is fetched from Etherscan."#,
            value_name = "PATH_OR_ADDRESS"
        )]
        path_or_address: String,
        #[clap(
            long,
            short,
            help = "The name to use for the generated interface",
            value_name = "NAME"
        )]
        name: Option<String>,
        #[clap(
            long,
            short,
            default_value = "^0.8.10",
            help = "Solidity pragma version.",
            value_name = "VERSION"
        )]
        pragma: String,
        #[clap(
            short,
            help = "The path to the output file.",
            long_help = "The path to the output file. If not specified, the interface will be output to stdout.",
            value_name = "PATH"
        )]
        output_location: Option<PathBuf>,
        #[clap(
            long,
            short,
            env = "ETHERSCAN_API_KEY",
            help = "etherscan API key",
            value_name = "KEY"
        )]
        etherscan_api_key: Option<String>,
        #[clap(flatten)]
        chain: ClapChain,
    },
    #[clap(name = "sig", alias = "si", about = "Get the selector for a function.")]
    Sig {
        #[clap(
            help = "The function signature, e.g. transfer(address,uint256).",
            value_name = "SIG"
        )]
        sig: String,
    },
    #[clap(
        name = "find-block",
        alias = "f",
        about = "Get the block number closest to the provided timestamp."
    )]
    FindBlock(FindBlockArgs),
    #[clap(alias = "com", about = "Generate shell completions script")]
    Completions {
        #[clap(arg_enum)]
        shell: clap_complete::Shell,
    },
    #[clap(
        name = "run",
        alias = "r",
        about = "Runs a published transaction in a local environment and prints the trace."
    )]
    Run(RunArgs),
}

#[derive(Debug, Parser)]
pub enum WalletSubcommands {
    #[clap(name = "new", alias = "n", about = "Create a new random keypair.")]
    New {
        #[clap(
            help = "If provided, then keypair will be written to an encrypted JSON keystore.",
            value_name = "PATH"
        )]
        path: Option<String>,
        #[clap(
            long,
            short,
            help = "Triggers a hidden password prompt for the JSON keystore.",
            conflicts_with = "unsafe-password",
            requires = "path"
        )]
        password: bool,
        #[clap(
            long,
            help = "Password for the JSON keystore in cleartext. This is UNSAFE to use and we recommend using the --password.",
            requires = "path",
            env = "CAST_PASSWORD",
            value_name = "PASSWORD"
        )]
        unsafe_password: Option<String>,
    },
    #[clap(name = "vanity", alias = "va", about = "Generate a vanity address.")]
    Vanity {
        #[clap(
            long,
            help = "Prefix for the vanity address.",
            required_unless_present = "ends-with",
            value_name = "HEX"
        )]
        starts_with: Option<String>,
        #[clap(long, help = "Suffix for the vanity address.", value_name = "HEX")]
        ends_with: Option<String>,
        #[clap(
            long,
            help = "Generate a vanity contract address created by the generated keypair with the specified nonce.",
            value_name = "NONCE"
        )]
        nonce: Option<u64>, /* 2^64-1 is max possible nonce per https://eips.ethereum.org/EIPS/eip-2681 */
    },
    #[clap(name = "address", aliases = &["a", "addr"], about = "Convert a private key to an address.")]
    Address {
        #[clap(flatten)]
        wallet: Wallet,
    },
    #[clap(name = "sign", alias = "s", about = "Sign a message.")]
    Sign {
        #[clap(help = "message to sign", value_name = "MESSAGE")]
        message: String,
        #[clap(flatten)]
        wallet: Wallet,
    },
    #[clap(name = "verify", alias = "v", about = "Verify the signature of a message.")]
    Verify {
        #[clap(help = "The original message.", value_name = "MESSAGE")]
        message: String,
        #[clap(help = "The signature to verify.", value_name = "SIGNATURE")]
        signature: String,
        #[clap(long, short, help = "The address of the message signer.", value_name = "ADDRESS")]
        address: String,
    },
}

pub fn parse_name_or_address(s: &str) -> eyre::Result<NameOrAddress> {
    Ok(if s.starts_with("0x") {
        NameOrAddress::Address(s.parse::<Address>()?)
    } else {
        NameOrAddress::Name(s.into())
    })
}

pub fn parse_block_id(s: &str) -> eyre::Result<BlockId> {
    Ok(match s {
        "earliest" => BlockId::Number(BlockNumber::Earliest),
        "latest" => BlockId::Number(BlockNumber::Latest),
        "pending" => BlockId::Number(BlockNumber::Pending),
        s if s.starts_with("0x") => BlockId::Hash(H256::from_str(s)?),
        s => BlockId::Number(BlockNumber::Number(u64::from_str(s)?.into())),
    })
}

fn parse_slot(s: &str) -> eyre::Result<H256> {
    Ok(if s.starts_with("0x") {
        let padded = format!("{:0>64}", s.strip_prefix("0x").unwrap());
        H256::from_str(&padded)?
    } else {
        H256::from_low_u64_be(u64::from_str(s)?)
    })
}

#[derive(Debug, Parser)]
#[clap(name = "cast", version = crate::utils::VERSION_MESSAGE)]
pub struct Opts {
    #[clap(subcommand)]
    pub sub: Subcommands,
}
