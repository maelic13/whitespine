#include "Constants.h"
#include "SearchOptions.h"

SearchOptions::SearchOptions() {
    fen = startPosition;
    playedMoves = std::vector<std::string>{};
    whiteTime = 0;
    whiteIncrement = 0;
    blackTime = 0;;
    blackIncrement = 0;
    depth = 2;
}

void SearchOptions::Reset() {
    ResetPosition();
    ResetSearchParameters();
}

void SearchOptions::ResetPosition() {
    fen = startPosition;
    playedMoves = std::vector<std::string>{};
}

void SearchOptions::ResetSearchParameters() {
    whiteTime = 0;
    whiteIncrement = 0;
    blackTime = 0;
    blackIncrement = 0;
    depth = 2;
}
