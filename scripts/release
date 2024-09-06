if [ $# -eq 0 ]; then
    echo "You must pass the version number."
    exit 1
fi

sed -i "3s/.*/version = \"$1\"/" Cargo.toml
cargo c
git add Cargo.*
git commit -m "release: version $1"
git tag -a "v$1" -m "Version $1"
