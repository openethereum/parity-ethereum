//! Calculates heapsize of util types.

use uint::*;
use hash::*;

known_heap_size!(0, H32, H64, H128, Address, H256, H264, H512, H520, H1024, H2048);
known_heap_size!(0, U128, U256);
