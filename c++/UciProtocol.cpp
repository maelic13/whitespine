#include <iostream>
#include <thread>

#include "Constants.h"
#include "UciProtocol.h"

UciProtocol::UciProtocol(std::thread &engineThread, Engine &engine)
        : engineThread(engineThread), engine(engine) {}

void UciProtocol::go() {
    engine.go = true;
}

void UciProtocol::stop() {
    engine.go = false;
}

void UciProtocol::quit() {
    engineThread.detach();
}

void UciProtocol::start() {
    std::string command;
    while (true) {
        getline(std::cin, command);
        std::cout << command << std::endl;

        if (command == "go") {
            engine.searchOptions.depth++;
            go();
        } else if (command == "stop") {
            stop();
        } else if (command == "quit") {
            quit();
            break;
        }
    }
}
