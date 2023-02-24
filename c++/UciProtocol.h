#ifndef ENGINE_UCIPROTOCOL_H
#define ENGINE_UCIPROTOCOL_H

#include <mutex>
#include <thread>

#include "EngineOptions.h"
#include "SearchOptions.h"

class UciProtocol {
public:
    UciProtocol(std::atomic_bool &go, std::atomic_bool &quit,
                EngineOptions &engineOptions, SearchOptions &searchOptions,
                std::mutex &m, std::condition_variable &cv);

    void UciLoop();

private:
    std::mutex &m;
    std::condition_variable &cv;

    std::atomic_bool &go;
    std::atomic_bool &quit;
    EngineOptions &engineOptions;
    SearchOptions &searchOptions;

    static void IsReady();

    static void Uci();

    void Go(const std::string &);

    void Stop();

    void Quit();

    static void SetOption(const std::string &);

    void UciNewGame();

    static void Position(const std::string &);
};

#endif //ENGINE_UCIPROTOCOL_H
