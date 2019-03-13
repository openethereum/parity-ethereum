# Copyright 2019 Parity Technologies (UK) Ltd.
# This file is part of Parity.
#
# Parity is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# Parity is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with Parity.  If not, see <http://www.gnu.org/licenses/>.

from itertools import islice

from parity import Parity

# Set up Parity
opts = ["--no-ipc", "--jsonrpc-apis=all", "--chain", "kovan"]
p = Parity(opts)

# Run a RPC query and print the results
query = "{\"method\":\"parity_versionInfo\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}"
print('version info:', p.rpc_query_sync(query))

# Subscribe to a websocket event
ws_query = "{\"method\":\"parity_subscribe\",\"params\":[\"parity_netPeers\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
sub = p.subscribe_ws(ws_query)

# Print the first 5 events received
for e in islice(sub.events, 5):
    print('subscription event', e)

# Unsubscribe to the event
sub.unsubscribe()

