#!/bin/bash

brew update
brew install coreutils
brew install rust
brew cask install xquartz
sudo util/build_prep/fetch_boost.sh
util/build_prep/macosx/build_qt.sh
git clone https://github.com/corrosion-rs/corrosion.git
cmake -Scorrosion -Bcorrosion_build -DCMAKE_BUILD_TYPE=Release
cmake --build corrosion_build --config Release
sudo cmake --install corrosion_build --config Release
