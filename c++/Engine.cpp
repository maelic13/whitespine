#include <chrono>
#include <iostream>

#include "Engine.h"

using namespace std::chrono_literals;

Engine::Engine(Options options) {
    m = &Options::m;
    cv = &Options::cv;

    go = options.go;
    quit = options.quit;

    engineOptions = options.engineOptions;
    searchOptions = options.searchOptions;
}

[[noreturn]] void Engine::start() {
    while (true) {
        std::unique_lock lk(m);
        cv.wait(lk, [&] { return go || quit; });

        if (quit) {
            lk.unlock();
            break;
        } else if (go) {
            search();
        }
    }
}

void Engine::search() {
    int depth = 0;
    while (go && !quit) {
        std::cout << "Calculating... Depth: " << depth << std::endl;
        depth++;
        std::this_thread::sleep_for(3s);
    }
}
