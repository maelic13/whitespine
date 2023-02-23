#include <chrono>
#include <iostream>
#include <thread>

#include "Constants.h"
#include "Engine.h"
#include "Options.h"
#include "UciProtocol.h"

int main() {
    std::cout << engineName << " " << engineVersion << " by " << engineAuthor << std::endl;

    Options options = Options();
    Engine engine = Engine(options);
    UciProtocol uciProtocol = UciProtocol(options);

    std::thread engineThread = std::thread(&Engine::start, &engine);
    uciProtocol.UciLoop();
    engineThread.join();

    return 0;
}
