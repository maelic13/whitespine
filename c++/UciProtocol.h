#ifndef ENGINE_UCIPROTOCOL_H
#define ENGINE_UCIPROTOCOL_H

#include <mutex>
#include <thread>

#include "SearchOptions.h"

class UciProtocol {
public:
    UciProtocol(SearchOptions &searchOptions, std::atomic_bool &go, std::atomic_bool &quit,
                std::mutex &m, std::condition_variable &cv);

    void Start();

    static void Uci();

    void Go();

    void Stop();

    void Quit();

private:
    std::atomic_bool &go;
    std::atomic_bool &quit;
    std::mutex &m;
    std::condition_variable &cv;
    SearchOptions &searchOptions;

    static void IsReady();
};

#endif //ENGINE_UCIPROTOCOL_H
