# third_party

Nothing large is vendored in Git.

## Valgrind (optional)

Build Valgrind 3.22.x locally under `third_party/valgrind` if the system package is unavailable:

```bash
# example outline — adjust URLs/versions as needed
curl -fsSL -o valgrind.tar.bz2 https://sourceware.org/pub/valgrind/valgrind-3.22.0.tar.bz2
tar -xjf valgrind.tar.bz2
cd valgrind-3.22.0
./configure --prefix="$PWD/../valgrind"
make -j"$(nproc)"
make install
```

Then:

```bash
export PATH="$PWD/third_party/valgrind/bin:$PATH"
cargo run -p cli -- trace-backend verify
```

`third_party/valgrind/` is gitignored.

## sysroot (optional, local only)

Some hosts lack CMake/Autotools system packages. Engineers may stage user-local tools under `third_party/sysroot/` (gitignored) or `$HOME/.local`. Scripts prepend these paths when present but never require them in Git.
