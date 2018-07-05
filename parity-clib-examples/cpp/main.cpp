// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <unistd.h>
#include <parity.h>

void on_restart(void*, const char*, size_t) {}

int main() {
    ParityParams cfg = { 0 };
    cfg.on_client_restart_cb = on_restart;

    const char* args[] = {"--no-ipc"};
    size_t str_lens[] = {8};
    if (parity_config_from_cli(args, str_lens, 1, &cfg.configuration) != 0) {
        return 1;
    }

    void* parity;
    if (parity_start(&cfg, &parity) != 0) {
        return 1;
    }

    const char* rpc = "{\"method\":\"parity_versionInfo\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}";
    size_t out_len = 256;
    char* out = (char*)malloc(out_len + 1);
    if (parity_rpc(parity, rpc, strlen(rpc), out, &out_len)) {
        return 1;
    }
    out[out_len] = '\0';
    printf("RPC output: %s", out);
    free(out);

    sleep(5);
    if (parity != NULL) {
        parity_destroy(parity);
    }

    return 0;
}
