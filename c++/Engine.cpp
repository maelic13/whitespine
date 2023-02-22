#include <chrono>
#include <iostream>

#include "Engine.h"

using namespace std::chrono_literals;

Engine::Engine(SearchOptions &searchOptions, std::atomic_bool &go, std::atomic_bool &quit,
               std::mutex &m, std::condition_variable &cv
               ) : searchOptions(searchOptions), go(go), quit(quit), m(m), cv(cv) {}

[[noreturn]] void Engine::start() {
    while (true) {
        std::unique_lock lk(m);
        cv.wait(lk, [&]{return go || quit;});

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
