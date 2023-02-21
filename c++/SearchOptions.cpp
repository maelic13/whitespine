#include "Constants.h"
#include "SearchOptions.h"

SearchOptions::SearchOptions() {
    fen = START_POSITION;
    playedMoves = std::vector<std::string>{};
    whiteTime = 0;
    whiteIncrement = 0;
    blackTime = 0;;
    blackIncrement = 0;
    depth = 2;
}

void SearchOptions::reset() {
    resetPosition();
    resetSearchParameters();
}

void SearchOptions::setPosition() {

}

void SearchOptions::setSearchParameters() {

}

void SearchOptions::resetPosition() {
    fen = START_POSITION;
    playedMoves = std::vector<std::string>{};
}

void SearchOptions::resetSearchParameters() {
    whiteTime = 0;
    whiteIncrement = 0;
    blackTime = 0;
    blackIncrement = 0;
    depth = 2;
}
