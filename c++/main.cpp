#include <chrono>
#include <iostream>
#include <thread>

#include "Constants.h"
#include "Engine.h"
#include "UciProtocol.h"

int main() {
    std::cout << engineName << " " << engineVersion << " by " << engineAuthor << std::endl;

    std::atomic_bool go = false;
    std::atomic_bool quit = false;
    std::mutex m;
    std::condition_variable cv;
    SearchOptions searchOptions = SearchOptions();

    Engine engine = Engine(searchOptions, go, quit, m, cv);
    UciProtocol uciProtocol = UciProtocol(searchOptions, go, quit, m, cv);

    std::thread engineThread = std::thread(&Engine::start, &engine);
    uciProtocol.Start();
    engineThread.join();

    return 0;
}
