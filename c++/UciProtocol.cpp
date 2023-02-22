#include <iostream>
#include <thread>

#include "Constants.h"
#include "UciProtocol.h"

UciProtocol::UciProtocol(
        SearchOptions &searchOptions, std::atomic_bool &go, std::atomic_bool &quit,
        std::mutex &m, std::condition_variable &cv
) : searchOptions(searchOptions), go(go), quit(quit), m(m), cv(cv) {}

void UciProtocol::Start() {
    std::string command;
    while (true) {
        getline(std::cin, command);

        if (command == "uci") Uci();
        if (command == "isready") IsReady();
        if (command == "go") Go();
        if (command == "stop") Stop();
        if (command == "quit") {
            Quit();
            break;
        }
    }
}

void UciProtocol::Uci() {
    std::cout << "id name " << engineName << " " << engineVersion << std::endl;
    std::cout << "id author " << engineAuthor << std::endl;
    std::cout << "uciok" << std::endl;
}

void UciProtocol::IsReady() {
    std::cout << "readyok" << std::endl;
}

void UciProtocol::Quit() {
    quit = true;
    cv.notify_one();
}

void UciProtocol::Go() {
    go = true;
    cv.notify_one();
}

void UciProtocol::Stop() {
    go = false;
}
