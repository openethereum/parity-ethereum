// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

#ifndef PARITY_HPP_INCLUDED
#define PARITY_HPP_INCLUDED
#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wc++98-compat"
#endif
#if __cplusplus < 201703L
#error                                                                         \
    "The Parity Ethereum C++ bindings require C++17 or newer.  Please upgrade your compiler, or specify the needed flags to enable C++17 support."
#endif
#include <cassert>
#include <exception>
#include <functional>
#include <memory>
#include <parity.h>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <vector>
static_assert(std::is_same<uintptr_t, size_t>::value,
              "Rust requires that uintptr_t and size_t are the same type");

namespace parity {
// avoid conflict with other Parity projects
namespace ethereum {
using namespace std::literals::string_literals;

class ParityException final : public std::exception {};

class ParityLogger final {
  friend class ParityEthereum;
  parity_logger *logger;

public:
  ParityLogger() = delete;
  explicit ParityLogger(const std::string &log_mode,
                        const std::string &log_file) noexcept
      : logger(nullptr) {
    parity_set_logger(log_mode.size() ? log_mode.data() : nullptr,
                      log_mode.size(),
                      log_file.size() ? log_file.data() : nullptr,
                      log_file.size(), &this->logger);
  }
  explicit ParityLogger(const ParityLogger &other) noexcept
      : logger(other.logger) {
    parity_logger_clone(logger);
  }
  ParityLogger &operator=(const ParityLogger &other) noexcept {
    if (logger != other.logger) {
      parity_logger_destroy(logger);
      logger = other.logger;
      parity_logger_clone(logger);
    }
    return *this;
  }
  ParityLogger(ParityLogger &&other) noexcept : logger(other.logger) {
    other.logger = nullptr;
  }
  ParityLogger &operator=(ParityLogger &&other) noexcept {
    if (logger != other.logger) {
      parity_logger_destroy(logger);
      logger = other.logger;
      other.logger = nullptr;
    }
    return *this;
  }
  ~ParityLogger() {
    parity_logger_destroy(logger);
    logger = nullptr;
  }
};

class ParityConfig final {
  friend class ParityEthereum;
  parity_config *config;

public:
  ParityConfig() = delete;
  ParityConfig(const ParityConfig &other) = delete;
  ParityConfig &operator=(const ParityConfig &other) = delete;
  explicit ParityConfig(const std::vector<std::string> &cli_args)
      : config(nullptr) {
    size_t const size = cli_args.size();
    std::vector<size_t> len_vecs;
    std::vector<char const *> args;
    len_vecs.reserve(size);
    args.reserve(size);
    for (const auto &i : cli_args) {
      len_vecs.push_back(i.size());
      args.push_back(i.data());
    }
    if (parity_config_from_cli(size ? args.data() : nullptr,
                               size ? len_vecs.data() : nullptr, size, &config))
      throw std::runtime_error(
          "failed to create Parity Ethereum configuration");
  }
  ParityConfig(ParityConfig &&other) noexcept : config(other.config) {
    other.config = nullptr;
  }
  ParityConfig &operator=(ParityConfig &&other) noexcept {
    if (this != &other) {
      assert((config == nullptr || config != other.config) &&
             "Different ParityConfig objects must wrap different parity_config "
             "pointers!");
      parity_config_destroy(config);
      config = other.config;
      other.config = nullptr;
    }
    return *this;
  }
  ~ParityConfig() { parity_config_destroy(config); }
};

using parity_rpc_callback =
    std::function<void(std::string_view const response)>;

namespace {
inline void parity_internal_rpc_callback(void *ud, const char *response,
                                         size_t len) noexcept {
  assert(ud != nullptr && "We always pass a valid userdata pointer");
  parity_rpc_callback *cb_ptr = static_cast<parity_rpc_callback *>(ud);
  (*cb_ptr)(std::string_view(response, len));
};

inline void parity_destructor_callback(void *ud) noexcept {
  assert(ud != nullptr && "We always pass a valid userdata pointer");
  delete static_cast<parity_rpc_callback *>(ud);
};
} // namespace

typedef std::unique_ptr<::parity_subscription,
                        decltype(parity_unsubscribe_ws) *>
    parity_subscription;
class ParityEthereum final {
  struct ::parity_ethereum *parity_ethereum_instance;
  parity_rpc_callback *callback;

public:
  ParityEthereum() = delete;
  ParityEthereum(const ParityEthereum &other) = delete;
  ParityEthereum &operator=(const ParityEthereum &other) = delete;
  ParityEthereum(ParityConfig config, ParityLogger logger,
                 parity_rpc_callback new_chain_spec_callback)
      : parity_ethereum_instance(nullptr),
        callback(new parity_rpc_callback(new_chain_spec_callback)) {
    struct ::ParityParams params = {
        config.config, parity_internal_rpc_callback,
        callback,      parity_destructor_callback,
        logger.logger,
    };
    logger.logger = nullptr;
    config.config = nullptr;
    if (parity_start(&params, &parity_ethereum_instance))
      throw std::runtime_error("Failed to start Parity Ethereum");
  }
  ParityEthereum(ParityEthereum &&other) noexcept
      : parity_ethereum_instance(other.parity_ethereum_instance),
        callback(other.callback) {
    other.parity_ethereum_instance = nullptr;
    other.callback = nullptr;
  }
  ParityEthereum &operator=(ParityEthereum &&other) noexcept {
    if (this != &other) {
      assert((parity_ethereum_instance == nullptr ||
              parity_ethereum_instance != other.parity_ethereum_instance) &&
             "ParityEthereum instances must each have a unique instance of "
             "Parity Ethereum");
      parity_destroy(parity_ethereum_instance);
      parity_ethereum_instance = other.parity_ethereum_instance;
      callback = other.callback;
      other.parity_ethereum_instance = nullptr;
      other.callback = nullptr;
    }
    return *this;
  }
  ~ParityEthereum() { parity_destroy(parity_ethereum_instance); }

  /// Perform an asychronous RPC request in a background thread.
  ///
  /// @param callback Callback to be called on a background thread.
  /// This must not throw an exception ― if it does, `std::terminate` is called.
  /// The callback’s destructor not called, and sizeof(callback) heap space is
  /// leaked.  This is a bug and will be fixed.  Note that when it is fixed, the
  /// destructor will be called on an arbitrary thread.
  void rpc(const std::string_view rpc_query, const std::size_t timeout_ms,
           parity_rpc_callback callback) const {
    if (::parity_rpc(this->parity_ethereum_instance, rpc_query.data(),
                     rpc_query.size(), timeout_ms, parity_internal_rpc_callback,
                     parity_destructor_callback,
                     new parity_rpc_callback(callback)))
      throw std::runtime_error("Parity RPC failed");
  }
  parity_subscription subscribe(const std::string_view buffer,
                                parity_rpc_callback callback) const {
    if (::parity_subscription *session = ::parity_subscribe_ws(
            this->parity_ethereum_instance, buffer.data(), buffer.size(),
            parity_internal_rpc_callback, new parity_rpc_callback(callback),
            parity_destructor_callback))
      return parity_subscription(session, &parity_unsubscribe_ws);
    else
      throw std::runtime_error("Failed to subscribe to websocket");
  }
};
} // namespace ethereum
} // namespace parity

#ifdef __clang__
#pragma clang diagnostic pop
#endif
#endif // include guard
