#!/usr/bin/env sh

main() {
    echo "Publishing the macro crate..."
    cd ./secretary-derive
    cargo publish
    
    echo "Publishing the main crate..."
    cd ../
    cargo publish
}

main "$@"