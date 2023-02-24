#ifndef ENGINE_ENGINE_H
#define ENGINE_ENGINE_H

#include "EngineOptions.h"
#include "SearchOptions.h"

class Engine {
public:
    explicit Engine(std::atomic_bool &go, std::atomic_bool &quit,
                    EngineOptions &engineOptions, SearchOptions &searchOptions,
                    std::mutex &m, std::condition_variable &cv);

    [[noreturn]] void start();

private:
    std::mutex &m;
    std::condition_variable &cv;

    std::atomic_bool &go;
    std::atomic_bool &quit;

    EngineOptions &engineOptions;
    SearchOptions &searchOptions;

    void search();
};

#endif //ENGINE_ENGINE_H
