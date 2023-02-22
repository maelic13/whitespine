#ifndef ENGINE_ENGINE_H
#define ENGINE_ENGINE_H

#include "SearchOptions.h"

class Engine {
public:
    explicit Engine(SearchOptions &searchOptions, std::atomic_bool &go, std::atomic_bool &quit,
                    std::mutex &m, std::condition_variable &cv);

    [[noreturn]] void start();

private:
    std::atomic_bool &go;
    std::atomic_bool &quit;
    std::mutex &m;
    std::condition_variable &cv;
    SearchOptions &searchOptions;

    void search();
};

#endif //ENGINE_ENGINE_H
