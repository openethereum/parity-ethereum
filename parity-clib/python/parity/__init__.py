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

from queue import Queue

# Rust extension module, see src/python.rs
import _parity


class _CallbackGenerator(object):
    """Thin wrapper around a Queue which can be passed as a callback to _parity.* functions"""

    def __init__(self):
        self.queue = Queue(maxsize=1)

    def __call__(self, value):
        self.queue.put(value)

    def __iter__(self):
        return self

    def __next__(self):
        return self.get()

    def get(self, block=True, timeout=None):
        """Get an element from the queue

        :param block: Should we block if no element available
        :param timeout: Time to wait for new element
        :return: Top item from the queue
        :raises: queue.Empty
        """
        return self.queue.get(block, timeout)

    def get_nowait(self):
        """Get an element from the queue, do not wait.

        Equivalent to `get(False)`

        :return: Top item from the queue
        :raises: queue.Empty
        """
        return self.queue.get_nowait()


class Subscription(object):
    """Encapsulates a subscription returned from subscribe_ws, allowing iteration over events and unsubscribing"""

    def __init__(self, sub, events):
        self._sub = sub
        self.events = events

    def unsubscribe(self):
        """Unsubscribe from the underlying subscription"""
        self._sub.unsubscribe()


class Parity(object):
    """Connection to Parity client"""

    def __init__(self, options, logger_mode='', logger_file=''):
        """Configure and start Parity

        :param options: Command line arguments to pass to Parity
        :param logger_mode: Logger options to pass to Parity
        :param logger_file: File to log to
        """
        config = _parity.config_from_cli(options)
        self.handle = _parity.build(config, logger_mode, logger_file)

    def rpc_query_async(self, query, cb, timeout_ms=1000):
        """Perform a RPC query, return immediately, cb will be invoked with the result asynchronously

        :param query: Query to perform
        :param cb: Callback to invoke with results
        :param timeout_ms: Timeout in milliseconds
        """
        _parity.rpc_query(self.handle, query, timeout_ms, cb)

    def rpc_query_sync(self, query, timeout_ms=1000):
        """Perform a RPC query and return the result synchronously

        :param query: Query to perform
        :param timeout_ms: Timeout in milliseconds
        :return: Result of the rpc call
        """
        cb = _CallbackGenerator()
        self.rpc_query_async(query, cb, timeout_ms)
        return next(cb)

    def subscribe_ws_cb(self, query, cb):
        """Subscribe to a websocket event, return immediately, cb will be invoked with events asynchronously

        :param query: Query to perform
        :param cb: Callback to invoke with events
        :return: Subscription handle
        """
        return _parity.subscribe_ws(self.handle, query, cb)

    def subscribe_ws(self, query):
        """Subscribe to a websocket event, return immediately, Subscription object can be iterated to receive events

        :param query: Query to perform
        :return: Subscription object which can be iterated over to receive events
        """
        cb = _CallbackGenerator()
        sub = self.subscribe_ws_cb(query, cb)
        return Subscription(sub, cb)
