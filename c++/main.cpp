#include <chrono>
#include <iostream>
#include <thread>

#include "Constants.h"
#include "Engine.h"
#include "EngineOptions.h"
#include "UciProtocol.h"

int main() {
    std::cout << engineName << " " << engineVersion << " by " << engineAuthor << std::endl;

    std::mutex m;
    std::condition_variable cv;

    std::atomic_bool go = false;
    std::atomic_bool quit = false;
    EngineOptions engineOptions = EngineOptions();
    SearchOptions searchOptions = SearchOptions();

    Engine engine = Engine(go, quit, engineOptions, searchOptions, m, cv);
    UciProtocol uciProtocol = UciProtocol(go, quit, engineOptions, searchOptions, m, cv);

    std::thread engineThread = std::thread(&Engine::start, &engine);
    uciProtocol.UciLoop();
    engineThread.join();

    return 0;
}
