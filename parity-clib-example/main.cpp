#include <cstddef>
#include <unistd.h>
#include <parity.h>

void on_restart(void*, const char*, size_t) {}

int main() {
    ParityParams cfg = { 0 };
    cfg.on_client_restart_cb = on_restart;

    const char* args[] = {"--light"};
    size_t str_lens[] = {7};
    if (parity_config_from_cli(args, str_lens, 1, &cfg.configuration) != 0) {
        return 1;
    }

    void* parity;
    if (parity_start(&cfg, &parity) != 0) {
        return 1;
    }

    sleep(5);
    if (parity != NULL) {
        parity_destroy(parity);
    }

    return 0;
}
