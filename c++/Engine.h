#ifndef ENGINE_ENGINE_H
#define ENGINE_ENGINE_H

#include "SearchOptions.h"

class Engine {
public:
    bool go;
    SearchOptions searchOptions;

    explicit Engine(SearchOptions searchOptions);

    void start();

private:
    void search();
};

#endif //ENGINE_ENGINE_H
