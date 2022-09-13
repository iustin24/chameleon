#!/usr/bin/env bash

# Script taken from https://github.com/epi052/feroxbuster
BASE_URL=https://github.com/iustin24/chameleon/releases/latest/download

MAC_ZIP=x86_64-macos-chameleon.zip
MAC_URL="$BASE_URL/$MAC_ZIP"

LIN32_ZIP=x86-linux-chameleon.zip
LIN32_URL="$BASE_URL/$LIN32_ZIP"

LIN64_ZIP=x86_64-linux-chameleon.zip
LIN64_URL="$BASE_URL/$LIN64_ZIP"

echo "[+] Installing Chameleon!"

if [ -f ~/.config/chameleon/config.toml ]; then
    echo "Config File Detected"
else
    wget https://raw.githubusercontent.com/iustin24/chameleon/master/config.toml -P ~/.config/chameleon/
fi
which unzip &>/dev/null
if [ "$?" != "0" ]; then
  echo "[ ] unzip not found, exiting. "
  exit -1
fi

if [[ "$(uname)" == "Darwin" ]]; then
  echo "[=] Found MacOS, downloading from $MAC_URL"
  curl -sLO "$MAC_URL"
  unzip -o "$MAC_ZIP" >/dev/null
  rm "$MAC_ZIP"
elif [[ "$(expr substr $(uname -s) 1 5)" == "Linux" ]]; then
  if [[ $(getconf LONG_BIT) == 32 ]]; then
    echo "Installing using the script is not supported for 32bit Linux."
  else
    echo "[=] Found 64-bit Linux, downloading from $LIN64_URL"
    curl -sLO "$LIN64_URL"
    unzip -o "$LIN64_ZIP" >/dev/null
    rm "$LIN64_ZIP"
  fi
fi

chmod +x ./chameleon

echo "[+] Downloading Chameleon Wordlists"

mkdir -p ~/.config/chameleon
which git &>/dev/null
if [ "$?" != "0" ]; then
  echo "[ ] Git not found - Could not download wordlists. "
else
  git clone https://github.com/iustin24/chameleon-wordlists/ ~/.config/chameleon/wordlists/
fi

echo "[+] Installed Chameleon version $(./chameleon -V)"