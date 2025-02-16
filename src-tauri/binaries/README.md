## Building macOS Universal Binary

macOS supports universal binaries, which run natively on x86_64 and arm64. In order to build such a binary you need the two separate builds plus the `lipo` command line tool. Then run the following:

```
lipo -create \
   -arch x86_64 squeezelite-x86_64-apple-darwin \
   -arch arm64 squeezelite-aarch64-apple-darwin \
   -output squeezelite-universal-apple-darwin
```