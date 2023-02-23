#ifndef ENGINE_OPTIONS_H
#define ENGINE_OPTIONS_H

#include <mutex>

#include "EngineOptions.h"
#include "SearchOptions.h"

class Options {
public:
    static std::mutex m;
    static std::condition_variable cv;

    std::atomic_bool go;
    std::atomic_bool quit;

    SearchOptions searchOptions;
    EngineOptions engineOptions;

    Options();
};

#endif //ENGINE_OPTIONS_H
