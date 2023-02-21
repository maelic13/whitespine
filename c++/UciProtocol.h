#ifndef ENGINE_UCIPROTOCOL_H
#define ENGINE_UCIPROTOCOL_H

#include <thread>

#include "Engine.h"
#include "UCIPROTOCOL.h"

class UciProtocol {
public:
    UciProtocol(std::thread &engineThread, Engine &engine);

    void start();

    void go();

    void stop();

    void quit();

private:
    std::thread &engineThread;
    Engine &engine;
};

#endif //ENGINE_UCIPROTOCOL_H
