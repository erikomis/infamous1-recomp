#!/bin/sh
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$here/infamous1-lle"
mkdir -p out/play
log="$PWD/out/play/$(date +%Y%m%d-%H%M%S).log"
echo "Abrindo inFAMOUS recompilado. Log: $log"
if ./run.sh > "$log" 2>&1; then
    exit 0
else
    status=$?
    echo "O jogo encerrou com código $status. Últimas mensagens:"
    tail -30 "$log"
    printf '\nPressione Enter para fechar.'
    read -r answer
    exit "$status"
fi
