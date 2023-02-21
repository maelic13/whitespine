#include <chrono>
#include <iostream>
#include <utility>

#include "Engine.h"

using namespace std;
using namespace chrono_literals;

Engine::Engine(SearchOptions searchOptions) : searchOptions(std::move(searchOptions)) {
    go = false;
}

void Engine::start() {
    while (true) {
        if (go) {
            search();
        }
    }
}

void Engine::search() {
    while (go) {
        this_thread::sleep_for(3s);
        cout << "Calculating... Depth: " << searchOptions.depth << endl;
    }
}
