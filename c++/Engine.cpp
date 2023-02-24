#include <chrono>
#include <iostream>

#include "Engine.h"

using namespace std::chrono_literals;

Engine::Engine(std::atomic_bool &go, std::atomic_bool &quit,
               EngineOptions &engineOptions, SearchOptions &searchOptions,
               std::mutex &m, std::condition_variable &cv)
        : go(go), quit(quit), engineOptions(engineOptions), searchOptions(searchOptions),
          m(m), cv(cv) {}

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
