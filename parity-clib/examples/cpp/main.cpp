// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

#ifdef __clang__
#pragma clang diagnostic error "-Wvexing-parse"
#endif
#include <chrono>
#include <cstdint>
#include <parity.h>
#include <parity.hpp>
#include <regex>
#include <string>
#include <thread>
namespace {
using namespace std::literals::string_literals;
using parity::ethereum::parity_subscription;
using parity::ethereum::ParityEthereum;

void parity_subscribe_to_websocket(ParityEthereum &ethereum);
void parity_rpc_queries(ParityEthereum &);
parity::ethereum::ParityEthereum parity_run(const std::vector<std::string> &);

constexpr uint32_t SUBSCRIPTION_ID_LEN = 18;
constexpr uint32_t TIMEOUT_ONE_MIN_AS_MILLIS = 60 * 1000;
enum class parity_callback_type : size_t {
  callback_rpc = 1,
  callback_ws = 2,
};

struct Callback {
  parity_callback_type type;
  std::uint64_t counter;
  void operator()(const std::string_view response) {
    switch (type) {
    case parity_callback_type::callback_rpc:
      counter -= 1;
      break;
    case parity_callback_type::callback_ws:
      std::match_results<std::string_view::iterator> results;
      std::regex is_subscription(
          R"(\{"jsonrpc":"2.0","result":"0[xX][a-fA-F0-9]{16}","id":1\})");
      if (std::regex_match(response.begin(), response.end(), results,
                           is_subscription)) {
        counter -= 1;
      }
      break;
    }
  }
};

// list of rpc queries
const std::vector<std::string> rpc_queries{
    R"({"method":"parity_versionInfo","params":[],"id":1,"jsonrpc":"2.0"})"s,
    R"({"method":"eth_getTransactionReceipt","params":["0x444172bef57ad978655171a8af2cfd89baa02a97fcb773067aef7794d6913fff"],"id":1,"jsonrpc":"2.0"})"s,
    R"({"method":"eth_estimateGas","params":[{"from":"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5"}],"id":1,"jsonrpc":"2.0"})"s,
    R"({"method":"eth_getBalance","params":["0x0066Dc48bb833d2B59f730F33952B3c29fE926F5"],"id":1,"jsonrpc":"2.0"})"s,
};

// list of subscriptions
const std::vector<std::string> ws_subscriptions{
    R"({"method":"parity_subscribe","params":["eth_getBalance",["0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826","latest"]],"id":1,"jsonrpc":"2.0"})"s,
    R"({"method":"parity_subscribe","params":["parity_netPeers"],"id":1,"jsonrpc":"2.0"})"s,
    R"({"method":"eth_subscribe","params":["newHeads"],"id":1,"jsonrpc":"2.0"})"s,
};

// callback that gets invoked upon an event
void callback(std::string_view buf) { (void)buf; }
} // namespace

int main() {
  using parity::ethereum::ParityEthereum;
  // run full-client
  {
    std::vector<std::string> cli_args{"--no-ipc"s, "--jsonrpc-apis=all"s,
                                      "--chain"s, "kovan"s};
    ParityEthereum parity = parity_run(cli_args);
    parity_rpc_queries(parity);
    parity_subscribe_to_websocket(parity);
  }

  // run light-client
  {
    std::vector<std::string> light_config = {
        "--no-ipc"s, "--light"s, "--jsonrpc-apis=all"s, "--chain"s, "kovan"s};
    ParityEthereum parity = parity_run(light_config);
    parity_rpc_queries(parity);
    parity_subscribe_to_websocket(parity);
    exit(1);
  }
  return 0;
}

namespace {
void parity_rpc_queries(ParityEthereum &parity) {
  Callback cb{parity_callback_type::callback_rpc, rpc_queries.size()};
  auto cb_func = std::function(cb);

  try {
    for (const auto &query : rpc_queries)
      parity.rpc(query, TIMEOUT_ONE_MIN_AS_MILLIS, cb_func);
  } catch (std::exception &exn) {
    std::cerr << exn.what() << std::endl;
    while (cb.counter != 0)
      ;
    throw;
  } catch (...) {
    while (cb.counter != 0)
      ;
    throw;
  }
  while (cb.counter != 0)
    ;
}

void parity_subscribe_to_websocket(ParityEthereum &parity) {
  // MUST outlive the std::vector below
  Callback cb{parity_callback_type::callback_ws, ws_subscriptions.size()};

  std::vector<parity_subscription> sessions;

  for (auto sub : ws_subscriptions)
    sessions.push_back(parity.subscribe(sub, cb));

  while (cb.counter != 0)
    ;
  std::this_thread::sleep_for(std::chrono::seconds(60));
}

parity::ethereum::ParityEthereum
parity_run(const std::vector<std::string> &cli_args) {
  parity::ethereum::ParityConfig config{cli_args};
  parity::ethereum::ParityLogger logger{"rpc=trace"s, ""s};
  return parity::ethereum::ParityEthereum{std::move(config), std::move(logger),
                                          callback};
}
} // namespace
