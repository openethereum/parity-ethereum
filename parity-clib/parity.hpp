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
#error C++17 is required
#endif
#include <cassert>
#include <exception>
#include <functional>
#include <memory>
#include <parity.h>
#include <stdexcept>
#include <string>
#include <vector>

namespace parity {
// avoid conflict with other Parity projects
namespace ethereum {

class ParityException final : public std::exception {};

class ParityLogger final {
  friend class ParityEthereum;
  parity_logger *logger;

public:
  ParityLogger(const std::string &log_mode, const std::string &log_file)
      : logger(nullptr) {
    if (parity_set_logger(log_mode.size() ? log_mode.data() : nullptr,
                          log_mode.size(),
                          log_file.size() ? log_file.data() : nullptr,
                          log_file.size(), &this->logger))
      throw std::runtime_error(std::string("Error creating logger"));
  }
  ParityLogger(ParityLogger &&other) noexcept {
    if (this != &other) {
      this->logger = other.logger;
      other.logger = nullptr;
    }
  }
  ParityLogger &operator=(ParityLogger &&other) noexcept {
    if (this != &other) {
      this->logger = other.logger;
      other.logger = nullptr;
    }
    return *this;
  }
  ~ParityLogger() {
    if (this->logger)
      assert(false && "ParityLogger objects must be moved into a ParityParams, "
                      "not destroyed");
  }
};

class ParityConfig final {
  friend class ParityEthereum;
  parity_config *config;

public:
  ParityConfig(const std::vector<std::string> &cli_args) : config(nullptr) {
    size_t const size = cli_args.size();
    std::vector<size_t> len_vecs((size));
    std::vector<char const *> args((size));
    for (const auto &i : cli_args) {
      len_vecs.push_back(i.size());
      args.push_back(i.data());
    }
    if (parity_config_from_cli(size ? args.data() : nullptr,
                               size ? len_vecs.data() : nullptr, size, &config))
      throw std::runtime_error(
          "failed to create Parity Ethereum configuration");
  }
  ParityConfig(ParityConfig &&other) noexcept {
    if (this != &other) {
      this->config = other.config;
      other.config = nullptr;
    }
  }
  ParityConfig &operator=(ParityConfig &&other) noexcept {
    if (this != &other) {
      this->config = other.config;
      other.config = nullptr;
    }
    return *this;
  }
  ~ParityConfig() {
    if (this->config)
      assert(false && "ParityConfig objects must be moved into a "
                      "ParityParams, not destroyed");
  }
};

typedef std::unique_ptr<::parity_subscription, decltype(parity_unsubscribe_ws)*> parity_subscription;
class ParityEthereum final {
  struct ::parity_ethereum *parity_ethereum_instance;
  std::unique_ptr<std::function<void(std::string_view)>> callback;

public:
  ParityEthereum(ParityConfig config, ParityLogger logger,
                 std::function<void(std::string_view)> new_chain_spec_callback)
      : parity_ethereum_instance(nullptr),
        callback(std::make_unique<decltype(new_chain_spec_callback)>(
            std::move(new_chain_spec_callback))) {
    struct ::ParityParams params = {
        config.config,
        [](void *custom, const char *new_chain, size_t new_chain_len) {
          auto view = std::string_view(new_chain, new_chain_len);
          static_cast<decltype(new_chain_spec_callback) *>(custom)->operator()(
              view);
        },
        this->callback.get(),
        logger.logger,
    };
    if (parity_start(&params, &this->parity_ethereum_instance))
      throw std::runtime_error("Failed to start Parity Ethereum");
  }
  ParityEthereum(ParityEthereum &&other) noexcept {
    if (this != &other) {
      this->parity_ethereum_instance = other.parity_ethereum_instance;
      other.parity_ethereum_instance = nullptr;
    }
  }
  ParityEthereum &operator=(ParityEthereum &&other) noexcept {
    if (this != &other) {
      this->parity_ethereum_instance = other.parity_ethereum_instance;
      other.parity_ethereum_instance = nullptr;
    }
    return *this;
  }
  ~ParityEthereum() { parity_destroy(this->parity_ethereum_instance); }

  /// Perform an asychronous RPC request in a background thread.
  ///
  /// @param callback Callback to be called on a background thread.
  /// This must not throw an exception ― if it does, `std::terminate` is called.
  /// The callback’s destructor not called, and sizeof(callback) heap space is
  /// leaked.  This is a bug and will be fixed.  Note that when it is fixed, the
  /// destructor will be called on an arbitrary thread.
  void
  rpc(const std::string_view rpc_query, const std::size_t timeout_ms,
      std::function<void(std::string_view const response)> callback) const {
    static constexpr auto raw_callback =
        [](void *ud, const char *response, size_t len) noexcept {
      decltype(callback) *cb_ptr = static_cast<decltype(callback) *>(ud);
      auto ptr = std::unique_ptr<decltype(callback)>(cb_ptr);
      (*ptr)(std::string_view(response, len));
    };
    if (::parity_rpc(this->parity_ethereum_instance, rpc_query.data(),
                     rpc_query.size(), timeout_ms, raw_callback,
                     std::make_unique<decltype(callback)>(std::move(callback))
                         .release()))
      throw std::runtime_error("Parity RPC failed");
  }
  parity_subscription subscribe(
      const std::string_view buffer,
      std::function<void(std::string_view const response)> callback) const {
    static constexpr auto raw_callback =
        [](void *ud, const char *response, size_t len) noexcept {
      decltype(callback) *cb_ptr = static_cast<decltype(callback) *>(ud);
      (*cb_ptr)(std::string_view(response, len));
    };
    if (::parity_subscription *session = ::parity_subscribe_ws(
            this->parity_ethereum_instance, buffer.data(), buffer.size(),
            raw_callback, std::make_unique<decltype(callback)>(std::move(callback)).release()))
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
