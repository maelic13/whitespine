#ifndef ENGINE_SEARCHOPTIONS_H
#define ENGINE_SEARCHOPTIONS_H

#include <string>
#include <vector>

class SearchOptions {
public:
    std::string fen;
    std::vector<std::string> playedMoves;
    int whiteTime;
    int whiteIncrement;
    int blackTime;
    int blackIncrement;
    int depth;

    SearchOptions();

    void Reset();

    void ResetPosition();

    void ResetSearchParameters();
};

#endif //ENGINE_SEARCHOPTIONS_H
