#include <iostream>
#include <thread>

#include "Constants.h"
#include "UciProtocol.h"

UciProtocol::UciProtocol(
        std::atomic_bool &go, std::atomic_bool &quit,
        EngineOptions &engineOptions, SearchOptions &searchOptions,
        std::mutex &m, std::condition_variable &cv)
        : go(go), quit(quit), engineOptions(engineOptions), searchOptions(searchOptions),
          m(m), cv(cv) {}

void UciProtocol::UciLoop() {
    std::string args, command, input;
    while (true) {
        getline(std::cin, input);
        auto pos = input.find(' ');
        command = input.substr(0, pos);
        if (pos != std::string::npos) args = input.substr(pos + 1);
        else args = "";

        if (command == "uci") Uci();
        if (command == "isready") IsReady();
        if (command == "go") Go(args);
        if (command == "stop") Stop();
        if (command == "setoption") SetOption(args);
        if (command == "ucinewgame") UciNewGame();
        if (command == "position") Position(args);
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

void UciProtocol::Go(const std::string &args) {
    std::cout << "go command called with arguments: " << args << std::endl;
    go = true;
    cv.notify_one();
}

void UciProtocol::Stop() {
    go = false;
}

void UciProtocol::SetOption(const std::string &) {
    std::cout << "No engine options currently supported." << std::endl;
}

void UciProtocol::UciNewGame() {
    searchOptions.ResetPosition();
}

void UciProtocol::Position(const std::string &args) {
    std::cout << "position command called with arguments: " << args << std::endl;
}
