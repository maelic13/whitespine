#include <chrono>
#include <iostream>
#include <thread>

#include "Engine.h"
#include "UciProtocol.h"

using namespace std;

int main() {
    Engine engine = Engine(SearchOptions());
    thread worker = thread(&Engine::start, &engine);
    UciProtocol uciProtocol = UciProtocol(worker, engine);

    uciProtocol.start();

    return 0;
}
